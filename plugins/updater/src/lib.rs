// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    async_runtime::Mutex,
    plugin::{Builder, TauriPlugin},
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
        let config = self.state::<Config>().inner();
        #[allow(unused_mut)]
        let mut builder = UpdaterBuilder::new(version, config.clone(), updater_config);
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
                builder = builder.app_image_path(appimage);
            }
        }
        builder
    }

    fn updater(&self) -> Result<Updater> {
        self.updater_builder().build()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R, Option<Config>> {
    Builder::<R, Option<Config>>::new("updater")
        .js_init_script(include_str!("api-iife.js").to_string())
        .setup(move |app, api| {
            app.manage(api.config().clone());
            app.manage(PendingUpdate(Default::default()));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::check,
            commands::download_and_install,
        ])
        .build()
}
