use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("event")
        .invoke_handler(tauri::generate_handler![
            commands::listen,
            commands::unlisten,
            commands::emit,
        ])
        .build()
}
