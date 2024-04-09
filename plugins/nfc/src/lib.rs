// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(mobile)]

use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

mod error;
mod models;

pub use error::{Error, Result};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.nfc";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_nfc);

/// Access to the nfc APIs.
pub struct Nfc<R: Runtime>(PluginHandle<R>);

#[derive(Deserialize)]
struct IsAvailableResponse {
    available: bool,
}

#[derive(Serialize)]
struct WriteRequest {
    records: Vec<NfcRecord>,
}

impl<R: Runtime> Nfc<R> {
    pub fn is_available(&self) -> crate::Result<bool> {
        self.0
            .run_mobile_plugin::<IsAvailableResponse>("isAvailable", ())
            .map(|r| r.available)
            .map_err(Into::into)
    }

    pub fn scan(&self, payload: ScanRequest) -> crate::Result<ScanResponse> {
        self.0
            .run_mobile_plugin("scan", payload)
            .map_err(Into::into)
    }

    pub fn write(&self, records: Vec<NfcRecord>) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("write", WriteRequest { records })
            .map_err(Into::into)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the NFC APIs.
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
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "NfcPlugin")?;
            #[cfg(target_os = "ios")]
            let handle = api.register_ios_plugin(init_plugin_nfc)?;
            app.manage(Nfc(handle));
            Ok(())
        })
        .build()
}
