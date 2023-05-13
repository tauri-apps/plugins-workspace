#![allow(unused_imports)]

use tauri::{AppHandle, Runtime};

#[cfg(feature = "allow-exit")]
#[tauri::command]
pub fn exit<R: Runtime>(app: AppHandle<R>, code: i32) {
    app.exit(code)
}

#[cfg(feature = "allow-restart")]
#[tauri::command]
pub fn restart<R: Runtime>(app: AppHandle<R>) {
    app.restart()
}
