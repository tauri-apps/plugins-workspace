// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use config::{Config, HttpAllowlistScope};
pub use reqwest as client;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

use std::{collections::HashMap, sync::Mutex};

mod commands;
mod config;
mod error;
mod scope;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;
type ClientId = u32;

pub struct Http<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    pub(crate) clients: Mutex<HashMap<ClientId, commands::Client>>,
    pub(crate) scope: scope::Scope,
}

impl<R: Runtime> Http<R> {}

pub trait HttpExt<R: Runtime> {
    fn http(&self) -> &Http<R>;
}

impl<R: Runtime, T: Manager<R>> HttpExt<R> for T {
    fn http(&self) -> &Http<R> {
        self.state::<Http<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R, Option<Config>> {
    Builder::<R, Option<Config>>::new("http")
        .invoke_handler(tauri::generate_handler![
            commands::create_client,
            commands::drop_client,
            commands::request
        ])
        .setup(|app, api| {
            let default_scope = HttpAllowlistScope::default();
            app.manage(Http {
                app: app.clone(),
                clients: Default::default(),
                scope: scope::Scope::new(
                    api.config()
                        .as_ref()
                        .map(|c| &c.scope)
                        .unwrap_or(&default_scope),
                ),
            });
            Ok(())
        })
        .build()
}
