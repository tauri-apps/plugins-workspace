// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[tauri::command]
pub fn platform() -> &'static str {
    crate::platform()
}

#[tauri::command]
pub fn version() -> String {
    crate::version().to_string()
}

#[tauri::command]
pub fn os_type() -> String {
    crate::type_().to_string()
}

#[tauri::command]
pub fn family() -> &'static str {
    crate::family()
}

#[tauri::command]
pub fn arch() -> &'static str {
    crate::arch()
}

#[tauri::command]
pub fn exe_extension() -> &'static str {
    crate::exe_extension()
}

#[tauri::command]
pub fn locale() -> Option<String> {
    crate::locale()
}

#[tauri::command]
pub fn hostname() -> String {
    crate::hostname()
}
