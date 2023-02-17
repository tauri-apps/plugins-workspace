// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOptions {
    pub quality: Option<u8>,
    #[serde(default)]
    pub allow_editing: bool,
    pub result_type: Option<String>,
    #[serde(default)]
    pub save_to_gallery: bool,
    pub width: Option<usize>,
    pub height: Option<usize>,
    #[serde(default)]
    pub correct_orientation: bool,
    pub source: Option<String>,
    pub direction: Option<String>,
    pub presentation_style: Option<String>,
    pub prompt_label_header: Option<String>,
    pub prompt_label_cancel: Option<String>,
    pub prompt_label_photo: Option<String>,
    pub prompt_label_picture: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    pub data: String,
    pub asset_url: Option<String>,
    pub format: String,
    #[serde(default)]
    pub saved: bool,
    pub exif: serde_json::Value,
}
