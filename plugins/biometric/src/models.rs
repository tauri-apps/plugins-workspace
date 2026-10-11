// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

/// Options for [`Biometric::authenticate`](crate::Biometric::authenticate).
#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthOptions {
    /// Enables authentication using the device's password. This feature is available on both Android and iOS.
    pub allow_device_credential: bool,
    /// Label for the Cancel button. This feature is available on both Android and iOS.
    pub cancel_title: Option<String>,
    /// Specifies the text displayed on the fallback button if biometric authentication fails. This feature is available iOS only.
    pub fallback_title: Option<String>,
    /// Title indicating the purpose of biometric verification. This feature is available Android only.
    pub title: Option<String>,
    /// SubTitle providing contextual information of biometric verification. This feature is available Android only.
    pub subtitle: Option<String>,
    /// Specifies whether additional user confirmation is required, such as pressing a button after successful biometric authentication. This feature is available Android only.
    pub confirmation_required: Option<bool>,
}

/// The kind of biometry hardware detected on the device.
#[derive(Debug, Clone, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum BiometryType {
    /// No biometry hardware is available, or it is not enrolled with the operating system.
    None = 0,
    /// Fingerprint authentication (Apple Touch ID or Android fingerprint).
    TouchID = 1,
    /// Face authentication (Apple Face ID or Android face authentication).
    FaceID = 2,
}

/// The result of [`Biometric::status`](crate::Biometric::status), describing whether biometric
/// authentication can currently be used.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    /// Whether the device can currently authenticate using biometrics.
    pub is_available: bool,
    /// The kind of biometry hardware detected on the device, even when [`Self::is_available`] is `false`.
    pub biometry_type: BiometryType,
    /// A human-readable reason why biometric authentication is unavailable. Only set when
    /// [`Self::is_available`] is `false`.
    pub error: Option<String>,
    /// A platform-specific error code describing why biometric authentication is unavailable.
    /// Only set when [`Self::is_available`] is `false`.
    pub error_code: Option<String>,
}
