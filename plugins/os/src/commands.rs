// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[tauri::command]
pub fn locale() -> Option<String> {
    crate::locale()
}

#[tauri::command]
pub fn hostname() -> String {
    crate::hostname()
}
