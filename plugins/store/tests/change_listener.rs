// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// `store://change` was emitted while the store was locked, and Rust listeners run inside the
// emit, so a listener that read the store (directly, or through a logger that reads its level
// from it) hung the call that changed it.

use std::{path::Path, sync::mpsc, thread, time::Duration};

use serde_json::{Value as JsonValue, json};
use tauri::{
    Listener,
    test::{MockRuntime, mock_builder, mock_context, noop_assets},
};
use tauri_plugin_store::{Store, StoreBuilder};

fn app() -> tauri::App<MockRuntime> {
    mock_builder()
        .plugin(tauri_plugin_store::Builder::new().build())
        .build(mock_context(noop_assets()))
        .unwrap()
}

/// Runs `change` on another thread and returns what a `store://change` listener read.
fn read_in_listener(
    app: &tauri::App<MockRuntime>,
    store: std::sync::Arc<Store<MockRuntime>>,
    change: impl FnOnce(&Store<MockRuntime>) + Send + 'static,
) -> Option<JsonValue> {
    let (tx, rx) = mpsc::channel();
    let reader = store.clone();
    app.listen("store://change", move |_| {
        let _ = tx.send(reader.get("zoom"));
    });
    thread::spawn(move || change(&store));
    rx.recv_timeout(Duration::from_secs(5))
        .expect("changing the store hung because the listener could not read it")
}

fn temp_store(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!("{name}-{}.json", std::process::id()));
    let _ = std::fs::remove_file(&path);
    path
}

fn cleanup(path: &Path) {
    let _ = std::fs::remove_file(path);
}

#[test]
fn a_change_listener_can_read_the_new_value() {
    let app = app();
    let path = temp_store("store-listener-set");
    let store = StoreBuilder::new(&app, &path)
        .disable_auto_save()
        .build()
        .unwrap();

    let read = read_in_listener(&app, store, |store| store.set("zoom", 1.25));
    cleanup(&path);
    assert_eq!(read, Some(json!(1.25)));
}

#[test]
fn a_reset_listener_reads_the_defaults() {
    let app = app();
    let path = temp_store("store-listener-reset");
    let store = StoreBuilder::new(&app, &path)
        .default("zoom", 1.0)
        .disable_auto_save()
        .build()
        .unwrap();
    store.set("zoom", 2.0);

    let read = read_in_listener(&app, store, |store| store.reset());
    cleanup(&path);
    assert_eq!(read, Some(json!(1.0)));
}
