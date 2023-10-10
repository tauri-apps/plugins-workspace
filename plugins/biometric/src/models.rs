// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthOptions {
    pub allow_device_credential: bool,
    /// iOS only.
    pub fallback_title: Option<String>,
    /// iOS only.
    pub cancel_title: Option<String>,
    /// Android only.
    pub title: Option<String>,
    /// Android only.
    pub subtitle: Option<String>,
    /// Android only.
    pub confirmation_required: Option<bool>,
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
