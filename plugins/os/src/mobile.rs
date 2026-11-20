// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, de::DeserializeOwned};
use tauri::{
    AppHandle, Runtime,
    plugin::{PluginApi, PluginHandle},
};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.os";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_os);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Os<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "OsPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_os)?;
    Ok(Os(handle))
}

/// Access to the device APIs of the os plugin.
pub struct Os<R: Runtime>(PluginHandle<R>);

#[derive(Deserialize)]
struct DeviceNameResponse {
    name: Option<String>,
}

impl<R: Runtime> Os<R> {
    /// Returns information about the device, see [`DeviceInfo`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
    pub fn device_info(&self) -> crate::Result<DeviceInfo> {
        self.0
            .run_mobile_plugin("getDeviceInfo", ())
            .map_err(Into::into)
    }

    /// Returns the name the user gave the device, e.g. `My Pixel`.
    ///
    /// - **Android:** the device name from the system settings (Android 7.1+). `None` on Android 7.0
    ///   or if the user never set one.
    /// - **iOS:** since iOS 16 this is a generic name such as `iPhone` unless the app has the
    ///   [`com.apple.developer.device-information.user-assigned-device-name`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.device-information.user-assigned-device-name)
    ///   entitlement, which Apple only grants on request.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
    pub fn device_name(&self) -> crate::Result<Option<String>> {
        self.0
            .run_mobile_plugin::<DeviceNameResponse>("getDeviceName", ())
            .map(|r| r.name)
            .map_err(Into::into)
    }

    /// Returns the battery level and charging state. Always `Some` on Android and iOS, but its
    /// fields can be `None` when the state is unknown (e.g. on the iOS simulator).
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
    pub fn battery_info(&self) -> crate::Result<Option<BatteryInfo>> {
        self.0
            .run_mobile_plugin::<BatteryInfo>("getBatteryInfo", ())
            .map(Some)
            .map_err(Into::into)
    }
}
