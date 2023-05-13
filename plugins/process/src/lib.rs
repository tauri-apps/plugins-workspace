use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("process")
        .invoke_handler(tauri::generate_handler![
            #[cfg(feature = "allow-exit")]
            commands::exit,
            #[cfg(feature = "allow-restart")]
            commands::restart
        ])
        .build()
}
