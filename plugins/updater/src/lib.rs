// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};

use tokio::sync::Mutex;

mod commands;
mod config;
mod error;
mod updater;

pub use config::Config;
pub use error::Error;
pub use updater::*;
pub type Result<T> = std::result::Result<T, Error>;

struct UpdaterState {
    target: Option<String>,
    config: Config,
}

struct PendingUpdate<R: Runtime>(Mutex<Option<UpdateResponse<R>>>);

#[derive(Default)]
pub struct Builder {
    target: Option<String>,
    installer_args: Option<Vec<String>>,
}

/// Extension trait to use the updater on [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`].
pub trait UpdaterExt<R: Runtime> {
    /// Gets the updater builder to manually check if an update is available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_updater::UpdaterExt;
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle();
    ///     tauri::async_runtime::spawn(async move {
    ///         let response = handle.updater().check().await;
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    fn updater(&self) -> updater::UpdateBuilder<R>;
}

impl<R: Runtime, T: Manager<R>> UpdaterExt<R> for T {
    fn updater(&self) -> updater::UpdateBuilder<R> {
        updater::builder(self.app_handle())
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target.replace(target.into());
        self
    }

    pub fn installer_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.installer_args
            .replace(args.into_iter().map(Into::into).collect());
        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R, Config> {
        let target = self.target;
        let installer_args = self.installer_args;
        PluginBuilder::<R, Config>::new("updater")
            .setup(move |app, api| {
                let mut config = api.config().clone();
                if let Some(installer_args) = installer_args {
                    config.installer_args = installer_args;
                }
                app.manage(UpdaterState { target, config });
                app.manage(PendingUpdate::<R>(Default::default()));
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                commands::check,
                commands::download_and_install
            ])
            .build()
    }
}
