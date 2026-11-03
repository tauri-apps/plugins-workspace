// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Only `PathAllowed` events were saved, so a path forbidden at runtime came back
// as allowed on the next start.

use tauri::{
    Manager,
    test::{MockRuntime, mock_builder, mock_context, noop_assets},
};
use tauri_plugin_fs::FsExt;

const IDENTIFIER: &str = "app.tauri.persisted-scope.forbid-test";

fn start() -> tauri::App<MockRuntime> {
    let mut context = mock_context(noop_assets());
    context.config_mut().identifier = IDENTIFIER.into();
    mock_builder()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_persisted_scope::init())
        .build(context)
        .unwrap()
}

#[test]
fn a_path_forbidden_at_runtime_stays_forbidden_after_a_restart() {
    let path = std::env::temp_dir().join("persisted-scope-forbid-test.txt");

    let app = start();
    let data_dir = app.path().app_data_dir().unwrap();
    let _ = std::fs::remove_dir_all(&data_dir);
    app.fs_scope().allow_file(&path).unwrap();
    app.fs_scope().forbid_file(&path).unwrap();
    drop(app);

    let allowed = start().fs_scope().is_allowed(&path);
    let _ = std::fs::remove_dir_all(&data_dir);
    assert!(
        !allowed,
        "the forbidden path should not be allowed after a restart"
    );
}
