// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(any(target_os = "linux", target_os = "freebsd"))]

use std::{
    fs,
    path::PathBuf,
    process::{Child, Command},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri_plugin_deep_link::DeepLinkExt;

struct Sandbox(PathBuf);

impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

struct ChildProcess(Child);

impl Drop for ChildProcess {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

#[test]
#[ignore = "requires xdg-mime and update-desktop-database"]
fn xdg_unregister() {
    if std::env::var_os("TAURI_XDG_TEST_CHILD").is_some() {
        exercise_registration();
        return;
    }

    let root = std::env::temp_dir().join(format!(
        "tauri-deep-link-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&root).unwrap();
    let sandbox = Sandbox(root);
    for directory in ["config", "data", "runtime", "system-config", "system-data"] {
        fs::create_dir(sandbox.0.join(directory)).unwrap();
    }
    // Exercise quoting with a real executable path containing a space.
    let executable_dir = sandbox.0.join("app directory");
    fs::create_dir(&executable_dir).unwrap();
    let executable = executable_dir.join("test-app");
    fs::copy(std::env::current_exe().unwrap(), &executable).unwrap();
    let mut child = ChildProcess(
        Command::new(&executable)
            .args(["--ignored", "--exact", "xdg_unregister", "--nocapture"])
            // Use xdg-utils' generic backend without the operator's desktop or bus.
            .env_clear()
            .env("PATH", std::env::var_os("PATH").unwrap())
            .env("HOME", &sandbox.0)
            .env("XDG_RUNTIME_DIR", sandbox.0.join("runtime"))
            .env("XDG_CONFIG_HOME", sandbox.0.join("config"))
            .env("XDG_DATA_HOME", sandbox.0.join("data"))
            .env("XDG_CONFIG_DIRS", sandbox.0.join("system-config"))
            .env("XDG_DATA_DIRS", sandbox.0.join("system-data"))
            .env("XDG_CURRENT_DESKTOP", "X-Generic")
            .env("TAURI_XDG_TEST_CHILD", "1")
            .spawn()
            .unwrap(),
    );
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if let Some(status) = child.0.try_wait().unwrap() {
            assert!(status.success(), "XDG registration child failed: {status}");
            break;
        }
        assert!(
            Instant::now() < deadline,
            "XDG registration child timed out"
        );
        thread::sleep(Duration::from_millis(20));
    }
}

fn exercise_registration() {
    let app = mock_builder()
        .plugin(tauri_plugin_deep_link::init())
        .build(mock_context(noop_assets()))
        .unwrap();
    let deep_link = app.deep_link();
    let config = PathBuf::from(std::env::var_os("XDG_CONFIG_HOME").unwrap());
    let data = PathBuf::from(std::env::var_os("XDG_DATA_HOME").unwrap());
    let mimeapps_path = config.join("mimeapps.list");
    fs::write(
        &mimeapps_path,
        "[Default Applications]\nx-scheme-handler/unrelated=other.desktop\n",
    )
    .unwrap();

    for scheme in ["tauri-test-first", "tauri-test-second", "tauri-test-first"] {
        deep_link.register(scheme).unwrap();
        assert!(deep_link.is_registered(scheme).unwrap());
    }

    let executable = tauri::utils::platform::current_exe().unwrap();
    let file_name = format!(
        "{}-handler.desktop",
        executable.file_name().unwrap().to_string_lossy()
    );
    let desktop_text = fs::read_to_string(data.join("applications").join(&file_name)).unwrap();
    // INI parsing consumes quotes; verify the desktop command's quoting in the file itself.
    assert_eq!(
        desktop_text.lines().find(|line| line.starts_with("Exec=")),
        Some(format!("Exec=\"{}\" %u", executable.display()).as_str())
    );
    let desktop = ini::Ini::load_from_str(&desktop_text).unwrap();
    let section = desktop.section(Some("Desktop Entry")).unwrap();
    let mimes: Vec<_> = section
        .get("MimeType")
        .unwrap()
        .split(';')
        .filter(|mime| !mime.is_empty())
        .collect();
    assert_eq!(mimes.len(), 2);
    assert!(mimes.contains(&"x-scheme-handler/tauri-test-first"));
    assert!(mimes.contains(&"x-scheme-handler/tauri-test-second"));

    deep_link.unregister("tauri-test-first").unwrap();
    assert!(!deep_link.is_registered("tauri-test-first").unwrap());
    assert!(deep_link.is_registered("tauri-test-second").unwrap());
    let after = fs::read_to_string(data.join("applications").join(&file_name)).unwrap();
    assert_eq!(
        after.lines().find(|line| line.starts_with("Exec=")),
        desktop_text.lines().find(|line| line.starts_with("Exec="))
    );
    let desktop = ini::Ini::load_from_str(&after).unwrap();
    assert_eq!(
        desktop
            .section(Some("Desktop Entry"))
            .unwrap()
            .get("MimeType"),
        Some("x-scheme-handler/tauri-test-second;")
    );
    let mut mimeapps = ini::Ini::load_from_file(&mimeapps_path).unwrap();
    let defaults = mimeapps.section(Some("Default Applications")).unwrap();
    assert!(defaults.get("x-scheme-handler/tauri-test-first").is_none());
    assert_eq!(
        defaults.get("x-scheme-handler/tauri-test-second"),
        Some(file_name.as_str())
    );
    assert_eq!(
        defaults.get("x-scheme-handler/unrelated"),
        Some("other.desktop")
    );

    // An app must not remove another handler that became the user's default.
    fs::copy(
        data.join("applications").join(&file_name),
        data.join("applications/other.desktop"),
    )
    .unwrap();
    mimeapps.with_section(Some("Default Applications")).set(
        "x-scheme-handler/tauri-test-second",
        format!("other.desktop;{file_name};last.desktop;"),
    );
    mimeapps
        .with_section(Some("Added Associations"))
        .set(
            "x-scheme-handler/tauri-test-second",
            format!("{file_name};other.desktop;"),
        )
        .set("x-scheme-handler/unrelated", "unrelated.desktop;");
    mimeapps.write_to_file(&mimeapps_path).unwrap();
    deep_link.unregister("tauri-test-second").unwrap();
    let mimeapps = ini::Ini::load_from_file(&mimeapps_path).unwrap();
    let defaults = mimeapps.section(Some("Default Applications")).unwrap();
    assert_eq!(
        defaults.get("x-scheme-handler/tauri-test-second"),
        Some("other.desktop;last.desktop;")
    );
    assert_eq!(
        defaults.get("x-scheme-handler/unrelated"),
        Some("other.desktop")
    );
    let added = mimeapps.section(Some("Added Associations")).unwrap();
    assert_eq!(
        added.get("x-scheme-handler/tauri-test-second"),
        Some("other.desktop;")
    );
    assert_eq!(
        added.get("x-scheme-handler/unrelated"),
        Some("unrelated.desktop;")
    );
    assert!(!deep_link.is_registered("tauri-test-second").unwrap());
    let desktop_path = data.join("applications").join(&file_name);
    let after = fs::read_to_string(&desktop_path).unwrap();
    assert_eq!(
        after.lines().find(|line| line.starts_with("Exec=")),
        desktop_text.lines().find(|line| line.starts_with("Exec="))
    );
    let desktop = ini::Ini::load_from_str(&after).unwrap();
    assert!(desktop
        .section(Some("Desktop Entry"))
        .unwrap()
        .get("MimeType")
        .is_none());
    let other = ini::Ini::load_from_file(data.join("applications/other.desktop")).unwrap();
    assert_eq!(
        other
            .section(Some("Desktop Entry"))
            .unwrap()
            .get("MimeType"),
        Some("x-scheme-handler/tauri-test-second;")
    );

    // Removing an association must not prevent a later registration.
    deep_link.register("tauri-test-first").unwrap();
    assert!(deep_link.is_registered("tauri-test-first").unwrap());
    deep_link.unregister("tauri-test-first").unwrap();
    deep_link.unregister("tauri-test-first").unwrap();
    assert!(!deep_link.is_registered("tauri-test-first").unwrap());

    // A removed desktop file may still have cached associations to withdraw.
    deep_link.register("tauri-test-first").unwrap();
    fs::remove_file(desktop_path).unwrap();
    deep_link.unregister("tauri-test-first").unwrap();
    assert!(!deep_link.is_registered("tauri-test-first").unwrap());
    fs::remove_file(&mimeapps_path).unwrap();
    deep_link.unregister("tauri-test-first").unwrap();
}
