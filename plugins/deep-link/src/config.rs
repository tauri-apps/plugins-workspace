// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// This module is also imported in build.rs!

use serde::{Deserialize, Deserializer};
use tauri_utils::config::DeepLinkProtocol;

#[derive(Deserialize, Clone)]
pub struct AssociatedDomain {
    #[serde(default = "default_schemes")]
    pub scheme: Vec<String>,

    #[serde(deserialize_with = "deserialize_associated_host")]
    pub host: String,

    #[serde(default)]
    pub path: Vec<String>,
    #[serde(default, alias = "path-pattern", rename = "pathPattern")]
    pub path_pattern: Vec<String>,
    #[serde(default, alias = "path-prefix", rename = "pathPrefix")]
    pub path_prefix: Vec<String>,
    #[serde(default, alias = "path-suffix", rename = "pathSuffix")]
    pub path_suffix: Vec<String>,
}

// TODO: Consider removing this in v3
fn default_schemes() -> Vec<String> {
    vec!["https".to_string(), "http".to_string()]
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

#[derive(Deserialize, Clone)]
pub struct Config {
    /// Mobile requires `https://<host>` urls.
    #[serde(default)]
    pub mobile: Vec<AssociatedDomain>,
    /// Desktop requires urls starting with `<scheme>://`.
    /// These urls are also active in dev mode on Android.
    #[allow(unused)] // Used in tauri-bundler
    #[serde(default)]
    pub desktop: DesktopProtocol,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
#[allow(unused)] // Used in tauri-bundler
pub enum DesktopProtocol {
    One(DeepLinkProtocol),
    List(Vec<DeepLinkProtocol>),
}

impl Default for DesktopProtocol {
    fn default() -> Self {
        Self::List(Vec::new())
    }
}

impl DesktopProtocol {
    #[allow(dead_code)]
    pub fn contains_scheme(&self, scheme: &String) -> bool {
        match self {
            Self::One(protocol) => protocol.schemes.contains(scheme),
            Self::List(protocols) => protocols
                .iter()
                .any(|protocol| protocol.schemes.contains(scheme)),
        }
    }

    #[allow(dead_code)]
    pub fn schemes(&self) -> Vec<String> {
        match self {
            Self::One(protocol) => protocol.schemes.clone(),
            Self::List(protocols) => protocols
                .iter()
                .flat_map(|protocol| protocol.schemes.clone())
                .collect(),
        }
    }
}
