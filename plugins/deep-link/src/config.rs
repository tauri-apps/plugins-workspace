// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This module is also imported in build.rs!

#![allow(dead_code)]

use serde::{Deserialize, Deserializer};

#[derive(Deserialize)]
pub struct AssociatedDomain {
    #[serde(deserialize_with = "deserialize_associated_host")]
    pub host: String,
    #[serde(default, alias = "path-prefix", rename = "pathPrefix")]
    pub path_prefix: Vec<String>,
}

fn deserialize_associated_host<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let host = String::deserialize(deserializer)?;
    if let Some((scheme, _host)) = host.split_once("://") {
        Err(serde::de::Error::custom(format!(
            "host `{host}` cannot start with a scheme, please remove the `{scheme}://` prefix"
        )))
    } else {
        Ok(host)
    }
}

#[derive(Deserialize)]
pub struct Config {
    pub domains: Vec<AssociatedDomain>,
}
