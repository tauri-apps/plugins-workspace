use tauri::{AppHandle, Runtime};

#[tauri::command]
pub fn version<R: Runtime>(app: AppHandle<R>) -> String {
    app.package_info().version.to_string()
}
