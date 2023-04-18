// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Clipboard;
#[cfg(mobile)]
use mobile::Clipboard;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the clipboard APIs.
pub trait ClipboardExt<R: Runtime> {
    fn clipboard(&self) -> &Clipboard<R>;
}

impl<R: Runtime, T: Manager<R>> crate::ClipboardExt<R> for T {
    fn clipboard(&self) -> &Clipboard<R> {
        self.state::<Clipboard<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("clipboard")
        .invoke_handler(tauri::generate_handler![commands::write, commands::read])
        .setup(|app, api| {
            #[cfg(mobile)]
            let clipboard = mobile::init(app, api)?;
            #[cfg(desktop)]
            let clipboard = desktop::init(app, api)?;
            app.manage(clipboard);
            Ok(())
        })
        .build()
}
