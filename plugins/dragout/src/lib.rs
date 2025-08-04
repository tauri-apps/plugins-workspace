#![allow(unexpected_cfgs)]

use tauri::{plugin::{Builder as PluginBuilder, TauriPlugin}, Runtime};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::new("dragout")
        .invoke_handler(tauri::generate_handler![native_drag_out])
        .setup(|_app, _api| {
            println!("[dragout] Plugin setup called");
            // Any platform specific init could go here
            #[cfg(target_os = "macos")]
            macos::init();
            Ok(())
        })
        .build()
}

#[tauri::command]
fn native_drag_out(archive_path: String, file_paths: Vec<String>, _target_dir: Option<String>) -> Result<(), String> {
    println!("[dragout] native_drag_out called: archive='{}' files={:?}", archive_path, file_paths);
    #[cfg(target_os = "macos")]
    {
        if let Some(first) = file_paths.first() {
            return crate::macos::start_drag(&archive_path, first);
        }
        return Err("file_paths empty".into());
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("native drag-out not implemented for this platform".into())
    }
}

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(all(target_os = "macos", feature = "objc2_backend"))]
pub mod macos_objc2;

#[cfg(not(target_os = "macos"))]
compile_error!("tauri-plugin-dragout currently supports only macOS");
