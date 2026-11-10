// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Loading a store took the plugin's stores lock and then the resource table,
// closing one took them the other way round, so doing both at once deadlocked.

// TODO: Fails in CI
#![cfg(not(windows))]

use std::{sync::mpsc, thread, time::Duration};

use tauri::test::{mock_builder, mock_context, noop_assets};
use tauri_plugin_store::{StoreBuilder, StoreExt};

#[test]
fn loading_and_closing_a_store_concurrently_does_not_deadlock() {
    let app = mock_builder()
        .plugin(tauri_plugin_store::Builder::new().build())
        .build(mock_context(noop_assets()))
        .unwrap();
    let path = std::env::temp_dir().join(format!("store-race-{}.json", std::process::id()));

    let (done, finished) = mpsc::channel();
    let loader = {
        let app = app.handle().clone();
        let path = path.clone();
        let done = done.clone();
        thread::spawn(move || {
            for _ in 0..5_000 {
                StoreBuilder::new(&app, &path)
                    .disable_auto_save()
                    .build()
                    .unwrap();
            }
            done.send(()).unwrap();
        })
    };
    let closer = {
        let app = app.handle().clone();
        thread::spawn(move || {
            for _ in 0..5_000 {
                if let Some(store) = app.get_store(&path) {
                    store.close_resource();
                }
            }
            done.send(()).unwrap();
        })
    };

    for _ in 0..2 {
        finished
            .recv_timeout(Duration::from_secs(30))
            .expect("loading and closing the same store deadlocked");
    }
    loader.join().unwrap();
    closer.join().unwrap();
}
