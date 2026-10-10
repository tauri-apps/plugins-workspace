// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;

/// HTTP plugin configuration, defined on the `plugins > http` object of your `tauri.conf.json`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    /// Whether the scope is checked on every hop of a redirect chain instead of only on the
    /// URL requested by the frontend. Defaults to `false`.
    ///
    /// When disabled, a server on an allowed origin can answer with a redirect to any other
    /// origin - an open redirect, or a server an attacker controls - and the plugin follows it,
    /// handing the webview a response from a URL the scope denies, such as a `localhost`
    /// service, an internal host or a cloud metadata endpoint.
    ///
    /// When enabled, a redirect to a URL that is not allowed by the scope fails with
    /// [`Error::UrlNotAllowed`](crate::Error::UrlNotAllowed) instead of being followed, so every
    /// redirect target must also be part of the scope. This is opt-in because it breaks
    /// applications that rely on being redirected outside of their configured scope.
    // TODO(v3): enforce the scope on redirects by default and remove this option
    #[serde(default)]
    pub scope_redirects: bool,
}

#[cfg(test)]
mod tests {
    use super::Config;

    #[test]
    fn deserializes_the_plugin_configuration() {
        // the plugin configuration is optional, and the scope check is opt-in
        let config: Option<Config> = serde_json::from_value(serde_json::Value::Null).unwrap();
        assert!(config.is_none());

        let config: Config = serde_json::from_str("{}").unwrap();
        assert!(!config.scope_redirects);

        let config: Config = serde_json::from_str(r#"{ "scopeRedirects": true }"#).unwrap();
        assert!(config.scope_redirects);

        // a typo must not silently disable the scope check
        assert!(serde_json::from_str::<Config>(r#"{ "scopeRedirect": true }"#).is_err());
    }
}
