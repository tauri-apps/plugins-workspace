// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

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
    pub max_attemps: Option<u32>,
    pub mode: Option<AuthMode>,
    pub cipher_key: Option<String>,
    pub cipher_data: Option<CipherData>,
}

#[derive(Debug, Clone, Serialize, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum AuthMode {
    PROMPT = 0,
    ENCRYPT = 1,
    DECRYPT = 2,
}

#[derive(Debug, Clone, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum BiometryType {
    None = 0,
    TouchID = 1,
    FaceID = 2,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub is_available: bool,
    pub biometry_type: BiometryType,
    pub error: Option<String>,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CipherData {
    pub data: Option<String>,
    pub initialization_vector: Option<String>,
}