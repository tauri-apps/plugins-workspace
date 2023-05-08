use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("app")
        .invoke_handler(tauri::generate_handler![
            commands::version,
            commands::name,
            commands::tauri_version,
            commands::show,
            commands::hide
        ])
        .build()
}
