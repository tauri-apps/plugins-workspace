// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Control the device flashlight (torch).
//!
//! - Supported platforms: Android and iOS.
//!
//! On Android the flashlight keeps working while the
//! [barcode scanner](https://docs.rs/tauri-plugin-barcode-scanner) plugin holds the camera.

#![cfg(mobile)]

use serde::Deserialize;
use tauri::{
    Manager, Runtime,
    plugin::{Builder, PluginHandle, TauriPlugin},
};

mod error;

pub use error::{Error, Result};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.flashlight";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_flashlight);

#[derive(Deserialize)]
struct AvailableResponse {
    available: bool,
}

#[derive(Deserialize)]
struct EnabledResponse {
    enabled: bool,
}

/// Access to the flashlight APIs.
pub struct Flashlight<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Flashlight<R> {
    /// Turns the flashlight on.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`] if the device has no flashlight, the flashlight is temporarily
    /// unavailable (e.g. the device is overheating) or the camera is held by another app.
    pub fn enable(&self) -> Result<()> {
        self.0.run_mobile_plugin("enable", ()).map_err(Into::into)
    }

    /// Turns the flashlight off.
    ///
    /// # Errors
    ///
    /// Same as [`Self::enable`].
    pub fn disable(&self) -> Result<()> {
        self.0.run_mobile_plugin("disable", ()).map_err(Into::into)
    }

    /// Flips the flashlight and returns its new state (`true` when it is now on).
    ///
    /// # Errors
    ///
    /// Same as [`Self::enable`].
    pub fn toggle(&self) -> Result<bool> {
        self.0
            .run_mobile_plugin::<EnabledResponse>("toggle", ())
            .map(|r| r.enabled)
            .map_err(Into::into)
    }

    /// Whether the device has a flashlight.
    pub fn is_available(&self) -> Result<bool> {
        self.0
            .run_mobile_plugin::<AvailableResponse>("isAvailable", ())
            .map(|r| r.available)
            .map_err(Into::into)
    }

    /// Whether the flashlight is currently on. Reads the hardware state, so it also reflects the flashlight
    /// being turned off by the system or another app.
    pub fn is_enabled(&self) -> Result<bool> {
        self.0
            .run_mobile_plugin::<EnabledResponse>("isEnabled", ())
            .map(|r| r.enabled)
            .map_err(Into::into)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the flashlight APIs.
pub trait FlashlightExt<R: Runtime> {
    /// Returns the [`Flashlight`] instance managed by the app.
    fn flashlight(&self) -> &Flashlight<R>;
}

impl<R: Runtime, T: Manager<R>> crate::FlashlightExt<R> for T {
    fn flashlight(&self) -> &Flashlight<R> {
        self.state::<Flashlight<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("flashlight")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "FlashlightPlugin")?;
            #[cfg(target_os = "ios")]
            let handle = api.register_ios_plugin(init_plugin_flashlight)?;
            app.manage(Flashlight(handle));
            Ok(())
        })
        .build()
}
