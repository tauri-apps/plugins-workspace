// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use tauri::{Manager, command};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RequestBody {
    id: i32,
    name: String,
}

#[command]
pub fn log_operation(event: String, payload: Option<String>) {
    log::info!("{} {:?}", event, payload);
}

#[command]
pub fn perform_request(endpoint: String, body: RequestBody) -> String {
    println!("{} {:?}", endpoint, body);
    "message response".into()
}

// Used by e2e tests to check that plugin operations release their native resources.
#[command]
pub fn webview_resource_names(webview: tauri::Webview) -> Vec<String> {
    webview
        .resources_table()
        .names()
        .map(|(_, name)| name.into_owned())
        .collect()
}
