// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(any(target_os = "linux", target_os = "freebsd"))]

use std::{
    env,
    path::Path,
    process::{Child, Command, Output, Stdio},
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};
use tauri::{
    test::{mock_builder, mock_context, noop_assets},
    RunEvent, WebviewWindowBuilder,
};

const TEST_NAME: &str = "dbus_handoff_and_release";
const ROLE_ENV: &str = "TAURI_SINGLE_INSTANCE_TEST_ROLE";
const ID_ENV: &str = "TAURI_SINGLE_INSTANCE_TEST_ID";
const ACQUIRED: &str = "single-instance name acquired";
const CHILD_ARGS: [&str; 5] = [
    "--ignored",
    "--exact",
    TEST_NAME,
    "--nocapture",
    "--test-threads=1",
];

struct TestChild(Option<Child>);

impl TestChild {
    fn spawn(role: &str, dbus_id: &str, cwd: &Path) -> Self {
        Self(Some(
            Command::new(env::current_exe().unwrap())
                .args(CHILD_ARGS)
                .env(ROLE_ENV, role)
                .env(ID_ENV, dbus_id)
                .current_dir(cwd)
                .stdout(Stdio::piped())
                .stderr(Stdio::inherit())
                .spawn()
                .expect("spawn single-instance test child"),
        ))
    }

    fn wait(mut self) -> Output {
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if self.0.as_mut().unwrap().try_wait().unwrap().is_some() {
                return self.0.take().unwrap().wait_with_output().unwrap();
            }
            assert!(Instant::now() < deadline, "test child timed out");
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

#[test]
#[ignore = "requires a session bus; run with dbus-run-session -- cargo test --test dbus -- --ignored"]
fn dbus_handoff_and_release() {
    assert!(
        env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some_and(|address| !address.is_empty()),
        "run this test inside dbus-run-session"
    );

    if let Ok(role) = env::var(ROLE_ENV) {
        assert!(matches!(role.as_str(), "secondary" | "probe"));
        let dbus_id = env::var(ID_ENV).unwrap();
        let _app = mock_builder()
            .plugin(
                tauri_plugin_single_instance::Builder::new()
                    .dbus_id(&dbus_id)
                    .build(),
            )
            .build(mock_context(noop_assets()))
            .unwrap();
        // A secondary instance must exit from plugin setup, before build returns.
        assert_eq!(role, "probe", "secondary instance acquired the bus name");
        let connection = zbus::blocking::Connection::session().unwrap();
        let bus = zbus::blocking::fdo::DBusProxy::new(&connection).unwrap();
        // The semver feature adds a suffix within this process-unique namespace.
        let prefix = format!("{dbus_id}.SingleInstance");
        let names: Vec<_> = bus
            .list_names()
            .unwrap()
            .into_iter()
            .filter(|name| name.as_str().starts_with(&prefix))
            .collect();
        assert_eq!(names.len(), 1, "probe must own the single-instance name");
        assert_eq!(
            bus.get_connection_unix_process_id(
                zbus::names::BusName::try_from(names[0].as_str()).unwrap()
            )
            .unwrap(),
            std::process::id()
        );
        println!("\n{ACQUIRED}");
        return;
    }

    let dbus_id = format!("org.tauri.SingleInstanceTest.p{}", std::process::id());
    let cwd = env::temp_dir().canonicalize().unwrap();
    let (tx, rx) = mpsc::channel();
    let app = mock_builder()
        .plugin(
            tauri_plugin_single_instance::Builder::new()
                .dbus_id(&dbus_id)
                .callback(move |_, args, cwd| tx.send((args, cwd)).unwrap())
                .build(),
        )
        .build(mock_context(noop_assets()))
        .unwrap();

    let secondary = TestChild::spawn("secondary", &dbus_id, &cwd).wait();
    assert!(secondary.status.success(), "secondary instance failed");
    let (args, forwarded_cwd) = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("secondary instance must invoke the primary callback");
    let expected_args: Vec<String> = std::iter::once(env::current_exe().unwrap().into_os_string())
        .chain(CHILD_ARGS.map(Into::into))
        .map(|arg| arg.into_string().unwrap())
        .collect();
    assert_eq!(args, expected_args);
    assert_eq!(forwarded_cwd, cwd.to_str().unwrap());
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));

    // Keep managed state alive so reacquisition proves the Exit hook released the
    // name, rather than merely observing the original connection being dropped.
    let retained_handle = app.handle().clone();
    let window = WebviewWindowBuilder::new(&app, "main", Default::default())
        .build()
        .unwrap();
    assert_eq!(
        app.run_return(move |_, event| {
            if let RunEvent::Ready = event {
                window.close().unwrap();
            }
        }),
        0
    );

    let probe = TestChild::spawn("probe", &dbus_id, &cwd).wait();
    assert!(probe.status.success(), "ownership probe failed");
    assert!(
        String::from_utf8(probe.stdout)
            .unwrap()
            .lines()
            .any(|line| line == ACQUIRED),
        "the bus name was not released on exit"
    );
    assert!(matches!(rx.try_recv(), Err(mpsc::TryRecvError::Empty)));
    drop(retained_handle);
}
