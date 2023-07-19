// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This module is also imported in build.rs!

#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct AssociatedDomain {
    pub host: String,
    #[serde(default, alias = "path-prefix", rename = "pathPrefix")]
    pub path_prefix: Vec<String>,
}

#[derive(Deserialize)]
pub struct Config {
    pub domains: Vec<AssociatedDomain>,
}
