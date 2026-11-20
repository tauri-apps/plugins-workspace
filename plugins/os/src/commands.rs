// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{AppHandle, Runtime};

use crate::{BatteryInfo, DeviceInfo, OsExt};

#[tauri::command]
pub fn locale() -> Option<String> {
    crate::locale()
}

#[tauri::command]
pub fn hostname() -> String {
    crate::hostname()
}

// async so the blocking mobile plugin call does not run on the main thread
#[tauri::command]
pub async fn device_info<R: Runtime>(app: AppHandle<R>) -> crate::Result<DeviceInfo> {
    app.os().device_info()
}

#[tauri::command]
pub async fn device_name<R: Runtime>(app: AppHandle<R>) -> crate::Result<Option<String>> {
    app.os().device_name()
}

#[tauri::command]
pub async fn battery_info<R: Runtime>(app: AppHandle<R>) -> crate::Result<Option<BatteryInfo>> {
    app.os().battery_info()
}
