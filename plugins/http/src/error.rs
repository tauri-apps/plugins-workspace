// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};
use url::Url;

/// Errors that can happen while using the HTTP plugin.
///
/// The error is serialized to the frontend as its [`Display`](std::fmt::Display) string.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// JSON serialization or deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// File system error, raised for instance when the cookie store file cannot be created or
    /// opened on startup.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Error from the underlying [`reqwest`] client, raised while building the client, sending the
    /// request or reading the response body.
    #[error(transparent)]
    Network(#[from] reqwest::Error),
    /// Error from the [`http`] crate, raised while building the response of a `data:` URL.
    #[error(transparent)]
    Http(#[from] http::Error),
    /// A header name given by the frontend is not a valid HTTP header name.
    #[error(transparent)]
    HttpInvalidHeaderName(#[from] http::header::InvalidHeaderName),
    /// A header value is not a valid HTTP header value.
    #[error(transparent)]
    HttpInvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    /// URL not allowed by the scope.
    ///
    /// Raised for the URL requested by the frontend, and - when the
    /// [`scope_redirects`](crate::Config::scope_redirects) configuration is enabled - for any
    /// redirect target that is not allowed by the scope.
    #[error("url not allowed on the configured scope: {0}")]
    UrlNotAllowed(Url),
    /// Failed to parse a URL.
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    /// HTTP method error.
    #[error(transparent)]
    HttpMethod(#[from] http::method::InvalidMethod),
    /// The requested URL uses an unsupported scheme. Only `http`, `https` and `data` are handled.
    #[error("scheme {0} not supported")]
    SchemeNotSupport(String),
    /// The request was aborted by the frontend before the response was received.
    #[error("Request canceled")]
    RequestCanceled,
    /// Error from the file system plugin.
    #[error(transparent)]
    FsError(#[from] tauri_plugin_fs::Error),
    /// The `data:` URL could not be processed.
    #[error("failed to process data url")]
    DataUrlError,
    /// The body of the `data:` URL could not be decoded into bytes.
    #[error("failed to decode data url into bytes")]
    DataUrlDecodeError,
    /// Error from the Tauri APIs, raised for instance while resolving a path or while reading the
    /// webview resource table.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    /// A response header value is not valid UTF-8, so it cannot be forwarded to the frontend.
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    /// The frontend requested dangerous client settings, but the `dangerous-settings` Cargo
    /// feature is not enabled.
    #[error("dangerous settings used but are not enabled")]
    DangerousSettings,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

/// Alias for a [`Result`](std::result::Result) with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
