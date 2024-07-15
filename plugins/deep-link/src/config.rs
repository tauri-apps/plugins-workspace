// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This module is also imported in build.rs!

use serde::{Deserialize, Deserializer};
use tauri_utils::config::DeepLinkProtocol;

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
    /// Mobile requires `https://<host>` urls.
    pub mobile: Vec<AssociatedDomain>,
    /// Desktop requires urls starting with `<scheme>://`.
    /// These urls are also active in dev mode on Android.
    #[allow(unused)] // Used in tauri-bundler
    pub desktop: DesktopProtocol,
}

#[derive(Deserialize)]
#[serde(untagged)]
#[allow(unused)] // Used in tauri-bundler
pub enum DesktopProtocol {
    One(DeepLinkProtocol),
    List(Vec<DeepLinkProtocol>),
}
