// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(mobile)]

use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

mod error;
mod models;

pub use error::{Error, Result};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.barcodescanner";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_barcode_scanner);

/// Access to the scanner APIs.
pub struct BarcodeScanner<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> BarcodeScanner<R> {}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the barcode scanner APIs.
pub trait BarcodeScannerExt<R: Runtime> {
    fn barcode_scanner(&self) -> &BarcodeScanner<R>;
}

impl<R: Runtime, T: Manager<R>> crate::BarcodeScannerExt<R> for T {
    fn barcode_scanner(&self) -> &BarcodeScanner<R> {
        self.state::<BarcodeScanner<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("barcode-scanner")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "BarcodeScannerPlugin")?;
            #[cfg(target_os = "ios")]
            let handle = api.register_ios_plugin(init_plugin_barcode_scanner)?;
            app.manage(BarcodeScanner(handle));
            Ok(())
        })
        .build()
}
