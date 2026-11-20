// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Information about the device the app is running on, as returned by [`Os::device_info`](crate::Os::device_info).
///
/// Every field is `None` when the platform cannot provide it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DeviceInfo {
    /// The device model identifier.
    ///
    /// - **Android:** [`Build.MODEL`](https://developer.android.com/reference/android/os/Build#MODEL), e.g. `Pixel 8`.
    /// - **iOS:** the machine identifier, e.g. `iPhone15,2`. On the simulator, the identifier of the simulated device.
    /// - **macOS:** the `hw.model` sysctl, e.g. `Mac14,2`.
    /// - **Linux:** the DMI product name, or the device tree model on boards without DMI (e.g. a Raspberry Pi).
    /// - **Windows:** the `SystemProductName` reported by the firmware.
    ///
    /// Desktop values come straight from the firmware and can be placeholders such as `To Be Filled By O.E.M.`.
    pub model: Option<String>,
    /// The device manufacturer.
    ///
    /// - **Android:** [`Build.MANUFACTURER`](https://developer.android.com/reference/android/os/Build#MANUFACTURER), e.g. `Google`.
    /// - **iOS** and **macOS:** always `Apple`.
    /// - **Linux:** the DMI system vendor.
    /// - **Windows:** the `SystemManufacturer` reported by the firmware.
    pub manufacturer: Option<String>,
    /// Whether the app is running on a simulator or emulator.
    ///
    /// - **Android:** a heuristic based on the build fingerprint and hardware name, which recognizes the
    ///   Android SDK emulator but not every third-party one.
    /// - **iOS:** whether the app was built for the simulator.
    /// - **Desktop:** always `None`. Virtual machines are not detected.
    pub is_virtual: Option<bool>,
    /// The total physical memory of the device in bytes.
    ///
    /// On Android this is the memory available to the kernel, which is slightly less than the
    /// installed RAM.
    pub total_memory: Option<u64>,
    /// The version of the system webview.
    ///
    /// - **Android:** the version of the WebView package, e.g. `120.0.6099.230`. `None` on Android 7 (API 24 and 25).
    /// - **iOS:** the `CFBundleVersion` of the WebKit framework, the same value as [`tauri::webview_version`](https://docs.rs/tauri/2/tauri/fn.webview_version.html).
    /// - **Desktop:** always `None`. Use [`tauri::webview_version`](https://docs.rs/tauri/2/tauri/fn.webview_version.html) instead.
    pub webview_version: Option<String>,
}

/// Battery information, as returned by [`Os::battery_info`](crate::Os::battery_info).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct BatteryInfo {
    /// The charge level, from `0.0` (empty) to `1.0` (full). `None` if it is unknown, such as on the iOS simulator.
    pub level: Option<f32>,
    /// Whether the device is plugged in and charging (or full). `None` if it is unknown, such as on the iOS simulator.
    pub is_charging: Option<bool>,
}
