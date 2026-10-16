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

    /// The custom URL schemes of this domain, i.e. every scheme but http and https.
    ///
    /// A domain may list both (e.g. `["https", "myapp"]`), so this is not the same as `!is_web_link()`.
    #[allow(dead_code)] // Used in build.rs
    pub fn custom_schemes(&self) -> impl Iterator<Item = &String> {
        self.scheme.iter().filter(|s| *s != "https" && *s != "http")
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
                eprintln!(
                    "Warning: AppLink uses only 'http' — allowed on Android but not secure for production."
                );
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
    if let Some(ref host) = opt
        && let Some((scheme, _)) = host.split_once("://")
    {
        return Err(serde::de::Error::custom(format!(
            "host `{host}` cannot start with a scheme, please remove the `{scheme}://` prefix"
        )));
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

/// An entry of the iOS `CFBundleURLTypes` Info.plist key.
#[allow(dead_code)] // Used in build.rs
#[derive(Debug, PartialEq, Eq)]
pub struct IosUrlType {
    /// `CFBundleURLName`
    pub name: String,
    /// `CFBundleURLSchemes`
    pub schemes: Vec<String>,
}

impl Config {
    /// The `com.apple.developer.associated-domains` entitlement values, given its current ones.
    ///
    /// The plugin only manages the `applinks:` entries: they are replaced by the configured app links,
    /// and every other entry (e.g. `webcredentials:` for passkeys) the app added is kept.
    #[allow(dead_code)] // Used in build.rs
    pub fn ios_associated_domains(&self, current: &[String]) -> Vec<String> {
        self.mobile
            .iter()
            .filter(|d| d.is_app_link())
            .filter_map(|d| d.host.as_ref())
            .map(|host| format!("applinks:{host}"))
            .chain(
                current
                    .iter()
                    .filter(|entry| !entry.starts_with("applinks:"))
                    .cloned(),
            )
            .collect()
    }

    /// The `CFBundleURLTypes` entries for the configured custom URL schemes.
    ///
    /// http and https are handled through app links (associated domains) on iOS,
    /// so they are never registered as URL schemes.
    #[allow(dead_code)] // Used in build.rs
    pub fn ios_url_types(&self) -> Vec<IosUrlType> {
        self.mobile
            .iter()
            .filter_map(|domain| {
                let schemes = domain.custom_schemes().cloned().collect::<Vec<_>>();
                let name = format!("$(PRODUCT_BUNDLE_IDENTIFIER).{}", schemes.first()?);
                Some(IosUrlType { name, schemes })
            })
            .collect()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn config(mobile: serde_json::Value) -> Config {
        serde_json::from_value(serde_json::json!({ "mobile": mobile })).unwrap()
    }

    #[test]
    fn ios_url_types_only_register_custom_schemes() {
        let config = config(serde_json::json!([
            { "scheme": ["myapp"] },
            { "scheme": ["https"], "host": "example.com" },
            { "scheme": ["https"], "host": "web.example.com", "appLink": false },
            { "host": "default-schemes.example.com" },
            { "scheme": ["https", "mixed"], "host": "mixed.example.com" },
            { "scheme": ["http", "mixed-web", "mixed-web2"], "host": "mixed-web.example.com", "appLink": false },
            { "scheme": [] },
        ]));

        assert_eq!(
            config.ios_url_types(),
            vec![
                IosUrlType {
                    name: "$(PRODUCT_BUNDLE_IDENTIFIER).myapp".into(),
                    schemes: vec!["myapp".into()],
                },
                IosUrlType {
                    name: "$(PRODUCT_BUNDLE_IDENTIFIER).mixed".into(),
                    schemes: vec!["mixed".into()],
                },
                IosUrlType {
                    name: "$(PRODUCT_BUNDLE_IDENTIFIER).mixed-web".into(),
                    schemes: vec!["mixed-web".into(), "mixed-web2".into()],
                },
            ]
        );
    }

    #[test]
    fn ios_associated_domains_only_include_app_links() {
        let config = config(serde_json::json!([
            { "scheme": ["myapp"] },
            { "scheme": ["https"], "host": "example.com" },
            { "scheme": ["https"], "host": "web.example.com", "appLink": false },
            { "scheme": ["https", "mixed"], "host": "mixed.example.com" },
        ]));

        assert_eq!(
            config.ios_associated_domains(&[]),
            vec!["applinks:example.com", "applinks:mixed.example.com"]
        );
    }

    #[test]
    fn ios_associated_domains_replace_app_links_and_keep_other_entries() {
        let config = config(serde_json::json!([
            { "host": "example.com" },
            { "host": "new.example.com" },
        ]));

        let current = [
            "applinks:example.com".to_string(),
            "webcredentials:example.com".to_string(),
            "applinks:removed.example.com".to_string(),
            "activitycontinuation:example.com".to_string(),
        ];

        assert_eq!(
            config.ios_associated_domains(&current),
            vec![
                "applinks:example.com",
                "applinks:new.example.com",
                "webcredentials:example.com",
                "activitycontinuation:example.com",
            ]
        );
    }

    #[test]
    fn ios_associated_domains_keep_other_entries_without_app_links() {
        let config = config(serde_json::json!([{ "scheme": ["myapp"] }]));

        assert_eq!(
            config.ios_associated_domains(&["webcredentials:example.com".to_string()]),
            vec!["webcredentials:example.com"]
        );
        assert!(
            config
                .ios_associated_domains(&["applinks:example.com".to_string()])
                .is_empty()
        );
    }
}
