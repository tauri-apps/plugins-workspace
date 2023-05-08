use tauri::{AppHandle, Runtime};

#[tauri::command]
pub fn exit<R: Runtime>(app: AppHandle<R>, code: i32) {
    app.exit(code)
}

#[tauri::command]
pub fn restart<R: Runtime>(app: AppHandle<R>) {
    app.restart()
}
