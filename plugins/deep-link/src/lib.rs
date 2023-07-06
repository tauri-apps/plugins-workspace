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
use desktop::DeepLink;
#[cfg(mobile)]
use mobile::DeepLink;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the deep-link APIs.
pub trait DeepLinkExt<R: Runtime> {
    fn deep_link(&self) -> &DeepLink<R>;
}

impl<R: Runtime, T: Manager<R>> crate::DeepLinkExt<R> for T {
    fn deep_link(&self) -> &DeepLink<R> {
        self.state::<DeepLink<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("deep-link")
        .js_init_script(include_str!("api-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![commands::execute])
        .setup(|app, api| {
            #[cfg(mobile)]
            let deep_link = mobile::init(app, api)?;
            #[cfg(desktop)]
            let deep_link = desktop::init(app, api)?;
            app.manage(deep_link);
            Ok(())
        })
        .build()
}
