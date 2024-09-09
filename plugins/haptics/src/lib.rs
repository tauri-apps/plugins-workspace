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
use desktop::Haptics;
#[cfg(mobile)]
use mobile::Haptics;

/* macro_rules! specta_builder {
    () => {
        ts::builder()
            .commands(collect_commands![
                commands::vibrate,
                commands::impact_feedback,
                commands::notification_feedback,
                commands::selection_feedback
            ])
            .header("// @ts-nocheck")
            .config(
                specta::ts::ExportConfig::default()
                    .bigint(specta::ts::BigIntExportBehavior::Number),
            )
    };
} */

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the haptics APIs.
pub trait HapticsExt<R: Runtime> {
    fn haptics(&self) -> &Haptics<R>;
}

impl<R: Runtime, T: Manager<R>> crate::HapticsExt<R> for T {
    fn haptics(&self) -> &Haptics<R> {
        self.state::<Haptics<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    /* let (invoke_handler, register_events) =
    specta_builder!().build_plugin_utils("haptics").unwrap(); */

    Builder::new("haptics")
        .invoke_handler(tauri::generate_handler![
            commands::vibrate,
            commands::impact_feedback,
            commands::notification_feedback,
            commands::selection_feedback
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let haptics = mobile::init(app, api)?;
            #[cfg(desktop)]
            let haptics = desktop::init(app, api)?;
            app.manage(haptics);
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
            .export_for_plugin("haptics")
            .expect("failed to export specta types");
    }
}
 */
