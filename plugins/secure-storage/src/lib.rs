// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(not(target_os = "android"))]
mod desktop;
#[cfg(target_os = "android")]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(not(target_os = "android"))]
pub use desktop::SecureStorage;
#[cfg(target_os = "android")]
pub use mobile::SecureStorage;

// TODO: Consider using a worker thread to handle caveats mentioned by keyring-rs

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the secure-storage APIs.
pub trait SecureStorageExt<R: Runtime> {
    fn secure_storage(&self) -> &SecureStorage<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SecureStorageExt<R> for T {
    fn secure_storage(&self) -> &SecureStorage<R> {
        self.state::<SecureStorage<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("secure-storage")
        .invoke_handler(tauri::generate_handler![
            commands::set_string,
            commands::get_string,
            commands::set_binary,
            commands::get_binary
        ])
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let secure_storage = mobile::init(app, api)?;
            #[cfg(not(target_os = "android"))]
            let secure_storage = desktop::init(app, api)?;
            app.manage(secure_storage);
            Ok(())
        })
        .build()
}
