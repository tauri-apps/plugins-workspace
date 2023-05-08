use std::path::PathBuf;

#[tauri::command]
pub fn platform() -> &'static str {
    crate::platform()
}

#[tauri::command]
pub fn version() -> String {
    crate::version().to_string()
}

#[tauri::command]
pub fn kind() -> String {
    crate::kind().to_string()
}

#[tauri::command]
pub fn arch() -> &'static str {
    crate::arch()
}

#[tauri::command]
pub fn tempdir() -> PathBuf {
    crate::tempdir()
}
