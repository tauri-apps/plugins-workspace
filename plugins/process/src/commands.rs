// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{AppHandle, Runtime};

#[tauri::command]
pub fn exit<R: Runtime>(app: AppHandle<R>, code: i32) {
    app.exit(code)
}

#[tauri::command]
pub fn restart<R: Runtime>(app: AppHandle<R>) {
    app.restart()
}
