// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Read and write to the system clipboard.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, RunEvent, Runtime,
};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;

pub use error::{Error, Result};

#[cfg(desktop)]
pub use desktop::Clipboard;
#[cfg(mobile)]
pub use mobile::Clipboard;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the clipboard APIs.
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
    Builder::new("clipboard-manager")
        .invoke_handler(tauri::generate_handler![
            commands::write_text,
            commands::read_text,
            commands::read_image,
            commands::write_image,
            commands::write_html,
            commands::clear
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let clipboard = mobile::init(app, api)?;
            #[cfg(desktop)]
            let clipboard = desktop::init(app, api)?;
            app.manage(clipboard);
            Ok(())
        })
        .on_event(|_app, _event| {
            #[cfg(desktop)]
            if let RunEvent::Exit = _event {
                _app.clipboard().cleanup();
            }
        })
        .build()
}
