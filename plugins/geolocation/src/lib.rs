// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

//use tauri_specta::*;

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
use desktop::Geolocation;
#[cfg(mobile)]
use mobile::Geolocation;

/* macro_rules! specta_builder {
    () => {
        ts::builder()
            .commands(collect_commands![
                commands::get_current_position,
                commands::watch_position,
                commands::clear_watch,
                commands::check_permissions,
                commands::request_permissions
            ])
            .header("// @ts-nocheck")
            .config(
                specta::ts::ExportConfig::default()
                    .bigint(specta::ts::BigIntExportBehavior::Number),
            )
    };
} */

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the geolocation APIs.
pub trait GeolocationExt<R: Runtime> {
    fn geolocation(&self) -> &Geolocation<R>;
}

impl<R: Runtime, T: Manager<R>> crate::GeolocationExt<R> for T {
    fn geolocation(&self) -> &Geolocation<R> {
        self.state::<Geolocation<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    /*     let (invoke_handler, register_events) =
    specta_builder!().build_plugin_utils("geolocation").unwrap(); */

    Builder::new("geolocation")
        .invoke_handler(tauri::generate_handler![
            commands::get_current_position,
            commands::watch_position,
            commands::clear_watch,
            commands::check_permissions,
            commands::request_permissions
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let geolocation = mobile::init(app, api)?;
            #[cfg(desktop)]
            let geolocation = desktop::init(app, api)?;
            app.manage(geolocation);
            Ok(())
        })
        .build()
}

/* #[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn export_types() {
        specta_builder!()
            .path("./guest-js/bindings.ts")
            .config(
                specta::ts::ExportConfig::default()
                    .formatter(specta::ts::formatter::prettier)
                    .bigint(specta::ts::BigIntExportBehavior::Number),
            )
            .export_for_plugin("geolocation")
            .expect("failed to export specta types");
    }
}
 */
