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
    #[serde(default, deserialize_with = "deserialize_associated_host")]
    pub host: Option<String>, // Optional custom uri schemes dont NEED a host (may have one still), but required for https/http schemes
    #[serde(default)]
    pub path: Vec<String>,
    #[serde(default, alias = "path-pattern", rename = "pathPattern")]
    pub path_pattern: Vec<String>,
    #[serde(default, alias = "path-prefix", rename = "pathPrefix")]
    pub path_prefix: Vec<String>,
    #[serde(default, alias = "path-suffix", rename = "pathSuffix")]
    pub path_suffix: Vec<String>,
    #[serde(default, alias = "app-link", rename = "appLink")]
    pub app_link: Option<bool>,
}

impl AssociatedDomain {
    /// Returns true if the domain uses http or https scheme.
    pub fn is_web_link(&self) -> bool {
        self.scheme.iter().any(|s| s == "https" || s == "http")
    }

    /// Returns true if the domain uses http or https scheme and has proper host configuration.
    pub fn is_app_link(&self) -> bool {
        self.app_link
            .unwrap_or_else(|| self.is_web_link() && self.host.is_some())
    }

    pub fn validate(&self) -> Result<(), String> {
        // Rule 1: All web links require a host.
        if self.is_web_link() && self.host.is_none() {
            return Err("Web link requires a host".into());
        }

        // Rule 2: If it's an App Link, ensure http(s) and host.
        if self.is_app_link() {
            if !self.is_web_link() {
                return Err("AppLink must be a valid web link (https/http + host)".into());
            }
            if self.scheme.iter().any(|s| s == "http") && !self.scheme.iter().any(|s| s == "https")
            {
                eprintln!("Warning: AppLink uses only 'http' — allowed on Android but not secure for production.");
            }
        }

        Ok(())
    }
}

// TODO: Consider removing this in v3
fn default_schemes() -> Vec<String> {
    vec!["https".to_string(), "http".to_string()]
}

fn deserialize_associated_host<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    if let Some(ref host) = opt {
        if let Some((scheme, _)) = host.split_once("://") {
            return Err(serde::de::Error::custom(format!(
                "host `{host}` cannot start with a scheme, please remove the `{scheme}://` prefix"
            )));
        }
    }
    Ok(opt)
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
