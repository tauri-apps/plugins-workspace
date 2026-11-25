// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Runs a real primary instance in the test process and launches the test binary again as the
//! second instance, to check that it is forwarded to the primary while the lock is held and that
//! it starts normally once [`tauri_plugin_single_instance::destroy`] or
//! [`tauri::AppHandle::cleanup_before_exit`] released it.
//!
//! On Linux the lock is a D-Bus name, so these tests need a session bus and are ignored by
//! default. Run them with `dbus-run-session -- cargo test --test instance_lock -- --ignored`.

#![cfg(not(any(target_os = "android", target_os = "ios")))]

use std::{
    env,
    process::{Child, Command, Output, Stdio},
    sync::{
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
    time::{Duration, Instant},
};
use tauri::{
    App, Manager, RunEvent, WebviewUrl, WebviewWindowBuilder,
    test::{MockRuntime, mock_builder, mock_context, noop_assets},
};

const CHILD_TEST: &str = "second_instance";
const ID_ENV: &str = "TAURI_SINGLE_INSTANCE_TEST_ID";
// An extra test name filter that matches nothing, so it is passed through as an argument.
const MARKER: &str = "single-instance-test-marker";
const STARTED: &str = "STARTED";

type Forwarded = mpsc::Receiver<(Vec<String>, String)>;

/// A bus name and file name friendly identifier, unique to this process and call.
fn unique_identifier() -> String {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    format!(
        "com.tauri.SingleInstanceTest.p{}.n{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

fn build_app(
    identifier: &str,
    callback: Option<mpsc::Sender<(Vec<String>, String)>>,
) -> App<MockRuntime> {
    let mut context = mock_context(noop_assets());
    context.config_mut().identifier = identifier.into();
    let mut plugin = tauri_plugin_single_instance::Builder::new();
    if let Some(tx) = callback {
        plugin = plugin.callback(move |_, args, cwd| {
            let _ = tx.send((args, cwd));
        });
    }
    mock_builder()
        .plugin(plugin.build())
        .build(context)
        .unwrap()
}

fn primary(identifier: &str) -> (App<MockRuntime>, Forwarded) {
    let (tx, rx) = mpsc::channel();
    (build_app(identifier, Some(tx)), rx)
}

/// The Windows plugin receives the second instance's arguments through a window owned by the
/// thread that set it up, so that thread must keep pumping messages while it waits.
#[cfg(windows)]
fn pump_messages() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, MSG, PM_REMOVE, PeekMessageW, TranslateMessage,
    };
    unsafe {
        let mut msg: MSG = std::mem::zeroed();
        while PeekMessageW(&mut msg, std::ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[cfg(not(windows))]
fn pump_messages() {}

/// Launches this test binary as a second instance of the app with the given identifier.
fn launch_second_instance(identifier: &str) -> Output {
    let mut child = Command::new(env::current_exe().unwrap())
        .args([
            "--include-ignored",
            "--exact",
            CHILD_TEST,
            "--nocapture",
            MARKER,
        ])
        .env(ID_ENV, identifier)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let deadline = Instant::now() + Duration::from_secs(30);
    loop {
        if child.try_wait().unwrap().is_some() {
            return child.wait_with_output().unwrap();
        }
        if Instant::now() >= deadline {
            kill(child);
            panic!("the second instance timed out");
        }
        pump_messages();
        thread::sleep(Duration::from_millis(10));
    }
}

fn kill(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

/// Asserts the second instance handed its arguments to the primary and exited.
fn assert_forwarded(output: Output, forwarded: &Forwarded) {
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        !stdout.contains(STARTED),
        "the second instance started: {stdout}"
    );
    let (args, cwd) = forwarded
        .recv_timeout(Duration::from_secs(5))
        .expect("the primary instance did not receive the arguments");
    assert_eq!(args.last().map(String::as_str), Some(MARKER), "{args:?}");
    assert_eq!(cwd, env::current_dir().unwrap().to_string_lossy());
}

/// Asserts the second instance found no running instance and started normally.
fn assert_started(output: Output, forwarded: &Forwarded) {
    assert!(output.status.success(), "{output:?}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains(STARTED),
        "the second instance did not start: {stdout}"
    );
    assert!(forwarded.try_recv().is_err());
}

/// The second instance. Only does something when launched by [`launch_second_instance`].
#[test]
#[ignore = "launched by the other tests"]
fn second_instance() {
    let Ok(identifier) = env::var(ID_ENV) else {
        return;
    };
    // A second instance exits inside `build` when it finds a running one.
    let app = build_app(&identifier, None);
    println!("{STARTED}");
    tauri_plugin_single_instance::destroy(&app);
}

#[test]
#[cfg_attr(target_os = "linux", ignore = "needs a D-Bus session bus")]
fn destroy_releases_the_lock() {
    let identifier = unique_identifier();
    let (app, forwarded) = primary(&identifier);

    assert_forwarded(launch_second_instance(&identifier), &forwarded);
    assert_forwarded(launch_second_instance(&identifier), &forwarded);

    tauri_plugin_single_instance::destroy(&app);
    assert_started(launch_second_instance(&identifier), &forwarded);

    // Destroying it again, and cleaning up afterwards, is a no-op.
    tauri_plugin_single_instance::destroy(&app);
    app.cleanup_before_exit();
}

#[test]
#[cfg_attr(target_os = "linux", ignore = "needs a D-Bus session bus")]
fn cleanup_before_exit_releases_the_lock() {
    let identifier = unique_identifier();
    let (app, forwarded) = primary(&identifier);

    assert_forwarded(launch_second_instance(&identifier), &forwarded);

    app.cleanup_before_exit();
    assert_started(launch_second_instance(&identifier), &forwarded);

    tauri_plugin_single_instance::destroy(&app);
}

#[test]
#[cfg_attr(target_os = "linux", ignore = "needs a D-Bus session bus")]
fn lock_is_held_during_the_exit_event() {
    let identifier = unique_identifier();
    let (app, forwarded) = primary(&identifier);
    WebviewWindowBuilder::new(&app, "main", WebviewUrl::default())
        .build()
        .unwrap();

    let (exit_tx, exit_rx) = mpsc::channel();
    let id = identifier.clone();
    app.run(move |app, event| match event {
        // The mock runtime exits once its last window is gone.
        RunEvent::Ready => app.get_webview_window("main").unwrap().destroy().unwrap(),
        RunEvent::Exit => {
            exit_tx.send(launch_second_instance(&id)).unwrap();
        }
        _ => {}
    });

    let during_exit = exit_rx.try_recv().expect("RunEvent::Exit was not emitted");
    assert_forwarded(during_exit, &forwarded);
    // `App::run` cleans up after the exit event.
    assert_started(launch_second_instance(&identifier), &forwarded);
}
