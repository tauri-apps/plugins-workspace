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
use desktop::Nfc;
#[cfg(mobile)]
use mobile::Nfc;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the nfc APIs.
pub trait NfcExt<R: Runtime> {
    fn nfc(&self) -> &Nfc<R>;
}

impl<R: Runtime, T: Manager<R>> crate::NfcExt<R> for T {
    fn nfc(&self) -> &Nfc<R> {
        self.state::<Nfc<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("nfc")
        .js_init_script(include_str!("api-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![commands::execute])
        .setup(|app, api| {
            #[cfg(mobile)]
            let nfc = mobile::init(app, api)?;
            #[cfg(desktop)]
            let nfc = desktop::init(app, api)?;
            app.manage(nfc);
            Ok(())
        })
        .build()
}
