// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(all(
    any(target_os = "linux", target_os = "freebsd"),
    not(feature = "semver")
))]

use std::{
    env, fs,
    path::PathBuf,
    process::{Child, Command, Output, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::test::{mock_builder, mock_context, noop_assets};

const TEST_NAME: &str = "dbus_setup_errors";
const ID_ENV: &str = "TAURI_DBUS_ERROR_ID";
const SETUP_ERROR: &str = "SETUP_ERROR:";

struct RejectCallback(mpsc::Sender<()>);

#[zbus::interface(name = "org.SingleInstance.DBus")]
impl RejectCallback {
    fn execute_callback(&self, _argv: Vec<String>, _cwd: String) -> zbus::fdo::Result<()> {
        self.0.send(()).unwrap();
        Err(zbus::fdo::Error::Failed(
            "test primary rejected delivery".into(),
        ))
    }
}

struct Sandbox(PathBuf);
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct TestChild(Option<Child>);
impl TestChild {
    fn spawn(id: &str, address: &str) -> Self {
        Self(Some(
            Command::new(env::current_exe().unwrap())
                .args(["--ignored", "--exact", TEST_NAME, "--nocapture"])
                .env(ID_ENV, id)
                .env("DBUS_SESSION_BUS_ADDRESS", address)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap(),
        ))
    }

    fn wait(mut self) -> Output {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if self.0.as_mut().unwrap().try_wait().unwrap().is_some() {
                return self.0.take().unwrap().wait_with_output().unwrap();
            }
            assert!(Instant::now() < deadline, "D-Bus test child timed out");
            thread::sleep(Duration::from_millis(10));
        }
    }
}
impl Drop for TestChild {
    fn drop(&mut self) {
        if let Some(child) = self.0.as_mut() {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

fn assert_setup_error(output: Output) -> String {
    assert_eq!(output.status.code(), Some(101));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains(SETUP_ERROR), "{stdout}");
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(
        stderr.contains("PluginInitialization(\"single-instance\""),
        "{stderr}"
    );
    stdout
}

#[test]
#[ignore = "requires an explicit dbus-run-session invocation"]
fn dbus_setup_errors() -> Result<(), tauri::Error> {
    if let Ok(id) = env::var(ID_ENV) {
        let result = mock_builder()
            .plugin(
                tauri_plugin_single_instance::Builder::new()
                    .dbus_id(id)
                    .build(),
            )
            .build(mock_context(noop_assets()));
        return match result {
            Err(tauri::Error::PluginInitialization(plugin, message)) => {
                assert_eq!(plugin, "single-instance");
                println!("{SETUP_ERROR} {message}");
                Err(tauri::Error::PluginInitialization(plugin, message))
            }
            _ => panic!("setup must return a plugin error or exit after successful handoff"),
        };
    }

    let address =
        env::var("DBUS_SESSION_BUS_ADDRESS").expect("run this test inside dbus-run-session");
    let id = format!("org.tauri.DBusErrorTest.p{}", std::process::id());
    let root = env::temp_dir().join(format!(
        "tauri-dbus-error-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let sandbox = Sandbox(root);
    let missing_address = format!("unix:path={}", sandbox.0.join("missing.sock").display());
    assert_setup_error(TestChild::spawn(&id, &missing_address).wait());
    assert_setup_error(TestChild::spawn(&id, "invalid-address").wait());
    assert_setup_error(TestChild::spawn("invalid dbus id", &address).wait());

    // A real endpoint rejects delivery immediately, without bus timeouts or startup races.
    let name = format!("{id}.SingleInstance");
    let path = format!("/{}", name.replace('.', "/"));
    let (tx, rx) = mpsc::channel();
    let owner = zbus::blocking::connection::Builder::session()
        .unwrap()
        .name(name.as_str())
        .unwrap()
        .serve_at(path.as_str(), RejectCallback(tx))
        .unwrap()
        .build()
        .unwrap();
    let failure = assert_setup_error(TestChild::spawn(&id, &address).wait());
    assert!(failure.contains("test primary rejected delivery"));
    rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(owner);

    // The successful primary and secondary paths still use the real plugin and bus.
    let (tx, rx) = mpsc::channel();
    let app = mock_builder()
        .plugin(
            tauri_plugin_single_instance::Builder::new()
                .dbus_id(format!("{id}.Success"))
                .callback(move |_, args, cwd| tx.send((args, cwd)).unwrap())
                .build(),
        )
        .build(mock_context(noop_assets()))?;
    let secondary = TestChild::spawn(&format!("{id}.Success"), &address).wait();
    assert!(secondary.status.success());
    let (args, cwd) = rx.recv_timeout(Duration::from_secs(1)).unwrap();
    assert_eq!(
        args,
        [
            env::current_exe().unwrap().to_str().unwrap(),
            "--ignored",
            "--exact",
            TEST_NAME,
            "--nocapture"
        ]
    );
    assert_eq!(cwd, env::current_dir().unwrap().to_str().unwrap());
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    tauri_plugin_single_instance::destroy(&app);
    Ok(())
}
