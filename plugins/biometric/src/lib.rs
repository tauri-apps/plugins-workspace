// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Biometric authentication: Touch ID and Face ID on iOS, fingerprint, face and
//! iris on Android, and Touch ID on macOS.

#![cfg(any(mobile, target_os = "macos"))]

#[cfg(mobile)]
use serde::Serialize;
#[cfg(mobile)]
use tauri::plugin::PluginHandle;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};
#[cfg(mobile)]
use tauri::Manager;

pub use models::*;

#[cfg(target_os = "macos")]
mod commands;
mod error;
#[cfg(target_os = "macos")]
mod macos;
mod models;

pub use error::{Error, Result};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.biometric";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_biometric);

/// Access to the biometric APIs.
#[cfg(mobile)]
pub struct Biometric<R: Runtime>(PluginHandle<R>);

#[cfg(mobile)]
#[derive(Serialize)]
struct AuthenticatePayload {
    reason: String,
    #[serde(flatten)]
    options: AuthOptions,
}

#[cfg(mobile)]
impl<R: Runtime> Biometric<R> {
    pub fn status(&self) -> crate::Result<Status> {
        self.0.run_mobile_plugin("status", ()).map_err(Into::into)
    }

    pub fn authenticate(&self, reason: String, options: AuthOptions) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("authenticate", AuthenticatePayload { reason, options })
            .map_err(Into::into)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the biometric APIs.
#[cfg(mobile)]
pub trait BiometricExt<R: Runtime> {
    fn biometric(&self) -> &Biometric<R>;
}

#[cfg(mobile)]
impl<R: Runtime, T: Manager<R>> crate::BiometricExt<R> for T {
    fn biometric(&self) -> &Biometric<R> {
        self.state::<Biometric<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let builder = Builder::new("biometric");

    // On macOS there is no native plugin to register: the commands below talk
    // to LocalAuthentication in-process.
    #[cfg(target_os = "macos")]
    let builder = builder.invoke_handler(tauri::generate_handler![
        commands::status,
        commands::authenticate
    ]);

    builder
        .setup(|_app, _api| {
            #[cfg(target_os = "android")]
            let handle = _api.register_android_plugin(PLUGIN_IDENTIFIER, "BiometricPlugin")?;
            #[cfg(target_os = "ios")]
            let handle = _api.register_ios_plugin(init_plugin_biometric)?;
            #[cfg(mobile)]
            _app.manage(Biometric(handle));
            Ok(())
        })
        .build()
}
