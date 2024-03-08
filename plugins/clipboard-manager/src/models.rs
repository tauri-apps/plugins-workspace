// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
#[cfg_attr(mobile, derive(Serialize))]
#[serde(rename_all = "camelCase")]
pub enum ClipKind {
    PlainText {
        label: Option<String>,
        text: String,
    },
    #[cfg(desktop)]
    Image {
        image: tauri::image::JsImage,
    },
    Html {
        html: String,
        alt_html: Option<String>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ClipboardContents {
    PlainText {
        text: String,
    },
    Image {
        bytes: Vec<u8>,
        width: usize,
        height: usize,
    },
}
