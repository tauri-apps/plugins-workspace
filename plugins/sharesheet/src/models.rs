// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SharesheetOptions {
  mime_type: Option<String>,
  title: Option<String>,
  thumbnail_uri: Option<String>,
}

