// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::ipc::Channel;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PingRequest {
    pub value: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PingResponse {
    pub value: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EventHandler {
    pub handler: Channel,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LastUrl {
    pub url: Option<String>,
}
