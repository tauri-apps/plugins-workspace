// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{AppHandleExt, StateFlags, WindowExt};
use tauri::{AppHandle, Manager, Runtime, command};

fn get_state_flags<R: Runtime>(
    app: &AppHandle<R>,
    flags: Option<u32>,
) -> std::result::Result<StateFlags, String> {
    let flags = if let Some(flags) = flags {
        StateFlags::from_bits(flags).ok_or_else(|| format!("Invalid state flags bits: {flags}"))?
    } else {
        let plugin_state = app.state::<crate::PluginState>();
        plugin_state.state_flags
    };
    Ok(flags)
}

#[command]
pub async fn save_window_state<R: Runtime>(
    app: AppHandle<R>,
    flags: Option<u32>,
) -> std::result::Result<(), String> {
    let flags = get_state_flags(&app, flags)?;
    let (tx, mut rx) = tauri::async_runtime::channel(1);
    let main_thread_app = app.clone();
    // Window getters can wait for the event loop. Keep the cache lock on that
    // thread too, so a resize event cannot block the loop waiting for the lock.
    app.run_on_main_thread(move || {
        let result = main_thread_app
            .save_window_state(flags)
            .map_err(|e| e.to_string());
        let _ = tx.try_send(result);
    })
    .map_err(|e| e.to_string())?;
    rx.recv()
        .await
        .ok_or_else(|| "Window state save task was dropped before completing".to_string())?
}

#[command]
pub async fn restore_state<R: Runtime>(
    app: AppHandle<R>,
    label: String,
    flags: Option<u32>,
) -> std::result::Result<(), String> {
    let flags = get_state_flags(&app, flags)?;
    app.get_webview_window(&label)
        .ok_or_else(|| format!("Couldn't find window with label: {label}"))?
        .restore_state(flags)
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[command]
pub fn filename<R: Runtime>(app: AppHandle<R>) -> String {
    app.filename()
}

#[cfg(test)]
mod tests {
    use super::save_window_state;
    use crate::{PluginState, StateFlags, WindowState, WindowStateCache};
    use std::{
        collections::HashMap,
        sync::{Arc, Mutex, mpsc},
        thread,
        time::{Duration, SystemTime, UNIX_EPOCH},
    };
    use tauri::{Manager, RunEvent, WebviewWindowBuilder};

    #[test]
    fn save_command_updates_windows_on_the_runtime_thread() {
        let app = tauri::test::mock_app();
        let runtime_thread = thread::current().id();
        let observed_threads = Arc::new(Mutex::new(Vec::new()));
        let observed = observed_threads.clone();
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let output_dir = std::env::temp_dir().join(format!(
            "tauri-window-state-test-{}-{suffix}",
            std::process::id()
        ));
        std::fs::create_dir_all(&output_dir).unwrap();
        let output_path = output_dir.join("state.json");

        // Manage these directly: plugin initialization/initial restore is outside
        // this assertion and would itself call map_label before app.run starts.
        app.manage(PluginState {
            state_flags: StateFlags::all(),
            filename: output_path.to_string_lossy().into_owned(),
            map_label: Some(Box::new(move |label| {
                observed.lock().unwrap().push(thread::current().id());
                label
            })),
        });
        app.manage(WindowStateCache(Arc::new(Mutex::new(HashMap::from([(
            "main".to_owned(),
            WindowState::default(),
        )])))));
        let window = WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let handle = app.handle().clone();
        let (result_tx, result_rx) = mpsc::channel();
        let (finished_tx, finished_rx) = mpsc::channel();
        let watchdog_window = window.clone();
        let watchdog = thread::spawn(move || {
            // Close the mock window if the save future never completes.
            // The command-level test runner also needs an external timeout.
            if finished_rx.recv_timeout(Duration::from_secs(10)).is_err() {
                let _ = watchdog_window.close();
            }
        });
        let mut launch = Some((handle, window, result_tx, output_path.clone()));

        // This must call run, rather than merely constructing mock_app:
        // MockRuntime executes worker Tasks inline before it starts running.
        app.run(move |_app, event| {
            if let RunEvent::Ready = event {
                let (handle, window, result_tx, output_path) = launch.take().unwrap();
                thread::spawn(move || {
                    let result = tauri::async_runtime::block_on(save_window_state(handle, None));
                    let saved = std::fs::read(&output_path);
                    std::fs::remove_file(&output_path).unwrap();
                    std::fs::create_dir(&output_path).unwrap();
                    let error = tauri::async_runtime::block_on(save_window_state(
                        window.app_handle().clone(),
                        None,
                    ));
                    let invalid_flags = tauri::async_runtime::block_on(save_window_state(
                        window.app_handle().clone(),
                        Some(u32::MAX),
                    ));
                    let _ = result_tx.send((result, saved, error, invalid_flags));
                    let _ = window.close();
                });
            }
        });
        let _ = finished_tx.send(());
        watchdog.join().unwrap();
        let result = result_rx.recv_timeout(Duration::from_secs(1));
        let observations = observed_threads.lock().unwrap().clone();
        std::fs::remove_dir_all(&output_dir).unwrap();

        let (saved_result, saved, error, invalid_flags) =
            result.expect("save command never completed");
        assert_eq!(saved_result, Ok(()));
        let state: serde_json::Value = serde_json::from_slice(&saved.unwrap()).unwrap();
        assert!(state.get("main").is_some());
        assert!(error.is_err(), "save command swallowed a filesystem error");
        assert_eq!(
            invalid_flags,
            Err(format!("Invalid state flags bits: {}", u32::MAX))
        );
        assert_eq!(observations, vec![runtime_thread, runtime_thread]);
    }
}
