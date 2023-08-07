// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/updater/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/updater)
//!
//! In-app updates for Tauri applications.
//!
//! - Supported platforms: Windows, Linux and macOS.crypted database and secure runtime.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use tauri::{
    async_runtime::Mutex,
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};

mod commands;
mod config;
mod error;
mod updater;

pub use config::Config;
pub use error::{Error, Result};
pub use updater::*;

struct PendingUpdate(Mutex<Option<Update>>);

/// Extension trait to use the updater on [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`].
pub trait UpdaterExt<R: Runtime> {
    /// Gets the updater builder to build and updater
    /// that can manually check if an update is available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_updater::UpdaterExt;
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle();
    ///     tauri::async_runtime::spawn(async move {
    ///         let response = handle.updater_builder().build().unwrap().check().await;
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    fn updater_builder(&self) -> UpdaterBuilder;

    /// Gets the updater to manually check if an update is available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_updater::UpdaterExt;
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle();
    ///     tauri::async_runtime::spawn(async move {
    ///         let response = handle.updater().unwrap().check().await;
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    fn updater(&self) -> Result<Updater>;
}

impl<R: Runtime, T: Manager<R>> UpdaterExt<R> for T {
    fn updater_builder(&self) -> UpdaterBuilder {
        let app = self.app_handle();
        let version = app.package_info().version.clone();
        let updater_config = app.config().tauri.bundle.updater.clone();
        let UpdaterState { config, target } = self.state::<UpdaterState>().inner();

        let mut builder = UpdaterBuilder::new(version, config.clone(), updater_config);

        if let Some(target) = target {
            builder = builder.target(target);
        }

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            let env = app.env();
            if let Some(appimage) = env.appimage {
                builder = builder.executable_path(appimage);
            }
        }

        builder
    }

    fn updater(&self) -> Result<Updater> {
        self.updater_builder().build()
    }
}

struct UpdaterState {
    target: Option<String>,
    config: Config,
}

#[derive(Default)]
pub struct Builder {
    target: Option<String>,
    installer_args: Option<Vec<String>>,
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
            .js_init_script(include_str!("api-iife.js").to_string())
            .setup(move |app, api| {
                let mut config = api.config().clone();
                if let Some(installer_args) = installer_args {
                    config.installer_args = installer_args;
                }
                app.manage(UpdaterState { target, config });
                app.manage(PendingUpdate(Default::default()));
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                commands::check,
                commands::download_and_install
            ])
            .build()
    }
}
