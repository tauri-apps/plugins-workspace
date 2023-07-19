// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code)]

use serde::Deserialize;

#[derive(Deserialize)]
pub struct AssociatedDomain {
    host: String,
    #[serde(default, alias = "path-prefix", rename = "pathPrefix")]
    path_prefix: Vec<String>,
}

#[derive(Deserialize)]
pub struct Config {
    domains: Vec<AssociatedDomain>,
}
