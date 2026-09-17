// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(any(target_os = "linux", target_os = "freebsd"))]

use std::{
    env, fs,
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    process::{Child, Command},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri_plugin_deep_link::{DeepLinkExt, Error};

const TEST_NAME: &str = "xdg_command_results";
const MODE_ENV: &str = "TAURI_XDG_COMMAND_MODE";

struct Sandbox(PathBuf);
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct TestChild(Child);
impl Drop for TestChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
#[ignore = "requires xdg-mime and update-desktop-database"]
fn xdg_command_results() {
    if let Ok(mode) = env::var(MODE_ENV) {
        exercise_commands(&mode);
        return;
    }

    let root = env::temp_dir().join(format!(
        "tauri-xdg-commands-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let sandbox = Sandbox(root);
    for mode in [
        "real",
        "update-failure",
        "register-failure",
        "query-failure",
        "unregister-failure",
    ] {
        let home = sandbox.0.join(mode);
        fs::create_dir(&home).unwrap();
        for directory in [
            "config",
            "data",
            "runtime",
            "system-config",
            "system-data",
            "bin",
        ] {
            fs::create_dir(home.join(directory)).unwrap();
        }
        for command in ["update-desktop-database", "xdg-mime"] {
            let path = home.join("bin").join(command);
            fs::write(
                &path,
                format!(
                    "#!/bin/sh\nprintf '%s\\n' '{command}' >> \"$HOME/calls\"\n\
                 if [ '{command}' = update-desktop-database ]; then\n\
                 [ \"$TAURI_XDG_COMMAND_MODE\" = update-failure ] && exit 7\n\
                 [ \"$TAURI_XDG_COMMAND_MODE\" = unregister-failure ] && exit 7\n\
                 else\n\
                 [ \"$TAURI_XDG_COMMAND_MODE\" = register-failure ] && exit 7\n\
                 printf '%s\\n' \"$TAURI_XDG_DESKTOP_FILE\"\n\
                 [ \"$TAURI_XDG_COMMAND_MODE\" = query-failure ] && exit 7\n\
                 fi\nexit 0\n"
                ),
            )
            .unwrap();
            fs::set_permissions(path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        let mut child = TestChild(
            Command::new(env::current_exe().unwrap())
                .args(["--ignored", "--exact", TEST_NAME, "--nocapture"])
                // Isolate real XDG tools from the operator's desktop, bus, and files.
                .env_clear()
                .env(
                    "PATH",
                    if mode == "real" {
                        env::var_os("PATH").unwrap()
                    } else {
                        home.join("bin").into_os_string()
                    },
                )
                .env("HOME", &home)
                .env("XDG_CONFIG_HOME", home.join("config"))
                .env("XDG_DATA_HOME", home.join("data"))
                .env("XDG_RUNTIME_DIR", home.join("runtime"))
                .env("XDG_CONFIG_DIRS", home.join("system-config"))
                .env("XDG_DATA_DIRS", home.join("system-data"))
                .env("XDG_CURRENT_DESKTOP", "X-Generic")
                .env(MODE_ENV, mode)
                .env("TAURI_XDG_DESKTOP_FILE", desktop_file_name())
                .spawn()
                .unwrap(),
        );
        let deadline = Instant::now() + Duration::from_secs(30);
        loop {
            if let Some(status) = child.0.try_wait().unwrap() {
                assert!(status.success(), "XDG child {mode} failed: {status}");
                break;
            }
            assert!(Instant::now() < deadline, "XDG child {mode} timed out");
            thread::sleep(Duration::from_millis(10));
        }
    }
}

fn desktop_file_name() -> String {
    format!(
        "{}-handler.desktop",
        tauri::utils::platform::current_exe()
            .unwrap()
            .file_name()
            .unwrap()
            .to_string_lossy()
    )
}

fn exercise_commands(mode: &str) {
    let app = mock_builder()
        .plugin(tauri_plugin_deep_link::init())
        .build(mock_context(noop_assets()))
        .unwrap();
    let deep_link = app.deep_link();
    let scheme = "tauri-command-test";
    if mode != "real" {
        let error = if mode == "query-failure" {
            deep_link.is_registered(scheme).unwrap_err()
        } else if mode == "unregister-failure" {
            // Registration leaves its desktop file when the cache update fails.
            assert!(matches!(deep_link.register(scheme), Err(Error::Io(_))));
            deep_link.unregister(scheme).unwrap_err()
        } else {
            deep_link.register(scheme).unwrap_err()
        };
        let Error::Io(error) = error else {
            panic!("command failure must use the existing I/O error variant")
        };
        assert_eq!(error.kind(), std::io::ErrorKind::Other);
        let command = if matches!(mode, "update-failure" | "unregister-failure") {
            "update-desktop-database"
        } else {
            "xdg-mime"
        };
        assert!(error.to_string().contains(command));
        assert!(error.to_string().contains('7'));
        let calls =
            fs::read_to_string(PathBuf::from(env::var_os("HOME").unwrap()).join("calls")).unwrap();
        let expected = match mode {
            "update-failure" => vec!["update-desktop-database"],
            "register-failure" => vec!["update-desktop-database", "xdg-mime"],
            "query-failure" => vec!["xdg-mime"],
            "unregister-failure" => vec!["update-desktop-database", "update-desktop-database"],
            _ => unreachable!(),
        };
        assert_eq!(calls.lines().collect::<Vec<_>>(), expected);
        return;
    }

    deep_link.register(scheme).unwrap();
    assert!(deep_link.is_registered(scheme).unwrap());
    let file_name = desktop_file_name();
    let other_file = format!("other-{file_name}");
    let applications = PathBuf::from(env::var_os("XDG_DATA_HOME").unwrap()).join("applications");
    // Use an independent handler: older xdg-utils cannot resolve a quoted Exec token.
    fs::write(
        applications.join(&other_file),
        format!("[Desktop Entry]\nType=Application\nName=Other handler\nExec=/bin/sh\nMimeType=x-scheme-handler/{scheme};\n"),
    )
    .unwrap();
    let mimeapps_path =
        PathBuf::from(env::var_os("XDG_CONFIG_HOME").unwrap()).join("mimeapps.list");
    let mut mimeapps = ini::Ini::load_from_file(&mimeapps_path).unwrap();
    mimeapps
        .with_section(Some("Default Applications"))
        .set(format!("x-scheme-handler/{scheme}"), &other_file);
    mimeapps.write_to_file(mimeapps_path).unwrap();
    let queried = Command::new("xdg-mime")
        .args(["query", "default", &format!("x-scheme-handler/{scheme}")])
        .output()
        .unwrap();
    assert!(queried.status.success());
    assert_eq!(
        String::from_utf8(queried.stdout)
            .unwrap()
            .trim_end_matches('\n'),
        other_file
    );
    assert!(!deep_link.is_registered(scheme).unwrap());
}
