// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Prompt the user for biometric authentication.
//!
//! - Supported platforms: Android and iOS.

#![cfg(mobile)]

use serde::Serialize;
use tauri::{
    plugin::{Builder, PluginHandle, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

mod error;
mod models;

pub use error::{Error, Result};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.biometric";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_biometric);

/// Access to the biometric APIs.
pub struct Biometric<R: Runtime>(PluginHandle<R>);

#[derive(Serialize)]
struct AuthenticatePayload {
    reason: String,
    #[serde(flatten)]
    options: AuthOptions,
}

impl<R: Runtime> Biometric<R> {
    /// Checks the device's availability and type of biometric authentication, as reported by the
    /// operating system. Errors if the underlying mobile plugin invocation fails.
    pub fn status(&self) -> crate::Result<Status> {
        self.0.run_mobile_plugin("status", ()).map_err(Into::into)
    }

    /// Prompts the user for biometric authentication using the system UI (Android
    /// `BiometricPrompt` or iOS `LocalAuthentication`), showing `reason` as the purpose of the
    /// request. Resolves once the user is authenticated and errors if authentication fails, is
    /// canceled, or the underlying mobile plugin invocation fails.
    pub fn authenticate(&self, reason: String, options: AuthOptions) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("authenticate", AuthenticatePayload { reason, options })
            .map_err(Into::into)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the biometric APIs.
pub trait BiometricExt<R: Runtime> {
    /// Returns the [`Biometric`] instance managed by the app.
    fn biometric(&self) -> &Biometric<R>;
}

impl<R: Runtime, T: Manager<R>> crate::BiometricExt<R> for T {
    fn biometric(&self) -> &Biometric<R> {
        self.state::<Biometric<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("biometric")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "BiometricPlugin")?;
            #[cfg(target_os = "ios")]
            let handle = api.register_ios_plugin(init_plugin_biometric)?;
            app.manage(Biometric(handle));
            Ok(())
        })
        .build()
}
