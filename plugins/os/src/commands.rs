use std::path::PathBuf;

use tauri::{AppHandle, Runtime, State};

use crate::{OperatingSystem, Result};

#[tauri::command]
pub fn platform<R: Runtime>(
    _app: AppHandle<R>,
    os: State<'_, OperatingSystem<R>>,
) -> Result<&'static str> {
    Ok(os.platform())
}

#[tauri::command]
pub fn version<R: Runtime>(
    _app: AppHandle<R>,
    os: State<'_, OperatingSystem<R>>,
) -> Result<String> {
    Ok(os.version().to_string())
}

#[tauri::command]
pub fn kind<R: Runtime>(_app: AppHandle<R>, os: State<'_, OperatingSystem<R>>) -> Result<String> {
    Ok(os.kind().to_string())
}

#[tauri::command]
pub fn arch<R: Runtime>(
    _app: AppHandle<R>,
    os: State<'_, OperatingSystem<R>>,
) -> Result<&'static str> {
    Ok(os.arch())
}

#[tauri::command]
pub fn tempdir<R: Runtime>(
    _app: AppHandle<R>,
    os: State<'_, OperatingSystem<R>>,
) -> Result<PathBuf> {
    Ok(os.tempdir())
}
