// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

#[derive(serde::Deserialize)]
pub struct Config {
    android: Vec<AndroidConfig>,
}

#[derive(serde::Deserialize)]
pub struct AndroidConfig {
    domain: String,
    #[serde(rename = "pathPrefix")]
    path_prefix: Option<String>,
}
