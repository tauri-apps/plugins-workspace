// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt, sync::Arc, time::Duration};

use http::{HeaderMap, Method};
use serde::Serialize;
use tauri::{AppHandle, Runtime};
use url::Url;

/// A transport extension invoked for every scoped HTTP(S) request.
///
/// Extensions are trusted native components registered with [`crate::Builder`].
/// They can validate request metadata before a client is built, configure the
/// final [`reqwest::ClientBuilder`], and classify transport errors without
/// changing the JavaScript API.
pub trait HttpTransportExtension<R: Runtime>: Send + Sync + 'static {
    /// Initializes the extension when the plugin is set up.
    fn setup(&self, _app: &AppHandle<R>) -> Result<(), ExtensionError> {
        Ok(())
    }

    /// Validates a request before any network client is built.
    fn validate(&self, _request: &RequestContext) -> Result<(), ExtensionError> {
        Ok(())
    }

    /// Configures the final request client after the plugin has applied its
    /// timeout, redirect, proxy, cookie, and feature-gated TLS settings.
    fn configure(
        &self,
        builder: reqwest::ClientBuilder,
        _request: &RequestContext,
    ) -> Result<reqwest::ClientBuilder, ExtensionError> {
        Ok(builder)
    }

    /// Maps an already-failed transport error to an extension-owned error.
    ///
    /// Returning `None` preserves the original [`reqwest::Error`].
    fn map_transport_error(
        &self,
        _error: &reqwest::Error,
        _request: &RequestContext,
    ) -> Option<ExtensionError> {
        None
    }
}

/// An extension-owned error value forwarded through the plugin IPC boundary.
///
/// The HTTP plugin treats the JSON value as opaque. This lets extensions keep
/// their own stable error contracts without adding extension-specific variants
/// to the plugin's error enum.
#[derive(Debug, Clone)]
pub struct ExtensionError(serde_json::Value);

impl ExtensionError {
    /// Creates an extension error from any serializable value.
    pub fn new(value: impl Serialize) -> Result<Self, serde_json::Error> {
        serde_json::to_value(value).map(Self)
    }

    /// Creates an extension error from an existing JSON value.
    pub fn from_value(value: serde_json::Value) -> Self {
        Self(value)
    }

    /// Returns the opaque JSON value supplied by the extension.
    pub fn value(&self) -> &serde_json::Value {
        &self.0
    }
}

impl fmt::Display for ExtensionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0.to_string())
    }
}

impl std::error::Error for ExtensionError {}

/// Immutable request metadata exposed to transport extensions.
///
/// The request body is never exposed. Extensions only receive whether a body
/// exists, along with the URL, method, filtered headers, and client options.
#[derive(Debug, Clone)]
pub struct RequestContext {
    method: Method,
    url: Url,
    headers: HeaderMap,
    has_body: bool,
    connect_timeout: Option<Duration>,
    max_redirections: Option<usize>,
    proxy_requested: bool,
    danger_accept_invalid_certs: bool,
    danger_accept_invalid_hostnames: bool,
}

impl RequestContext {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        method: Method,
        url: Url,
        headers: HeaderMap,
        has_body: bool,
        connect_timeout: Option<Duration>,
        max_redirections: Option<usize>,
        proxy_requested: bool,
        danger_accept_invalid_certs: bool,
        danger_accept_invalid_hostnames: bool,
    ) -> Self {
        Self {
            method,
            url,
            headers,
            has_body,
            connect_timeout,
            max_redirections,
            proxy_requested,
            danger_accept_invalid_certs,
            danger_accept_invalid_hostnames,
        }
    }

    pub fn method(&self) -> &Method {
        &self.method
    }

    pub fn url(&self) -> &Url {
        &self.url
    }

    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    pub fn has_body(&self) -> bool {
        self.has_body
    }

    pub fn connect_timeout(&self) -> Option<Duration> {
        self.connect_timeout
    }

    pub fn max_redirections(&self) -> Option<usize> {
        self.max_redirections
    }

    pub fn proxy_requested(&self) -> bool {
        self.proxy_requested
    }

    pub fn danger_accept_invalid_certs(&self) -> bool {
        self.danger_accept_invalid_certs
    }

    pub fn danger_accept_invalid_hostnames(&self) -> bool {
        self.danger_accept_invalid_hostnames
    }
}

pub(crate) type ExtensionList<R> = Arc<Vec<Arc<dyn HttpTransportExtension<R>>>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_error_preserves_structured_json() {
        let error = ExtensionError::new(serde_json::json!({
            "code": "POLICY_REJECTED",
            "host": "example.test"
        }))
        .unwrap();

        assert_eq!(
            error.to_string(),
            r#"{"code":"POLICY_REJECTED","host":"example.test"}"#
        );
        assert_eq!(error.value()["code"], "POLICY_REJECTED");
    }

    #[test]
    fn request_context_exposes_owned_request_metadata() {
        let context = RequestContext::new(
            Method::POST,
            Url::parse("https://example.test/path").unwrap(),
            HeaderMap::new(),
            true,
            Some(Duration::from_secs(2)),
            Some(3),
            true,
            false,
            false,
        );

        assert_eq!(context.method(), Method::POST);
        assert_eq!(context.url().host_str(), Some("example.test"));
        assert!(context.has_body());
        assert_eq!(context.connect_timeout(), Some(Duration::from_secs(2)));
        assert_eq!(context.max_redirections(), Some(3));
        assert!(context.proxy_requested());
        assert!(!context.danger_accept_invalid_certs());
        assert!(!context.danger_accept_invalid_hostnames());
    }
}
