use std::path::PathBuf;

use reqwest::Url;
use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Network(#[from] reqwest::Error),
    /// URL not allowed by the scope.
    #[error("url not allowed on the configured scope: {0}")]
    UrlNotAllowed(Url),
    /// Path not allowed by the scope.
    #[error("path not allowed on the configured scope: {0}")]
    PathNotAllowed(PathBuf),
    /// Client with specified ID not found.
    #[error("http client dropped or not initialized")]
    HttpClientNotInitialized,
    /// HTTP method error.
    #[error(transparent)]
    HttpMethod(#[from] http::method::InvalidMethod),
    /// Failed to serialize header value as string.
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
