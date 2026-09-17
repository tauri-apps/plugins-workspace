// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};
use url::Url;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Network(#[from] reqwest::Error),
    #[error(transparent)]
    Extension(#[from] crate::ExtensionError),
    #[error(transparent)]
    Http(#[from] http::Error),
    #[error(transparent)]
    HttpInvalidHeaderName(#[from] http::header::InvalidHeaderName),
    #[error(transparent)]
    HttpInvalidHeaderValue(#[from] http::header::InvalidHeaderValue),
    /// URL not allowed by the scope.
    #[error("url not allowed on the configured scope: {0}")]
    UrlNotAllowed(Url),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    /// HTTP method error.
    #[error(transparent)]
    HttpMethod(#[from] http::method::InvalidMethod),
    #[error("scheme {0} not supported")]
    SchemeNotSupport(String),
    #[error("Request canceled")]
    RequestCanceled,
    #[error(transparent)]
    FsError(#[from] tauri_plugin_fs::Error),
    #[error("failed to process data url")]
    DataUrlError,
    #[error("failed to decode data url into bytes")]
    DataUrlDecodeError,
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("dangerous settings used but are not enabled")]
    DangerousSettings,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Extension(error) => error.value().serialize(serializer),
            _ => serializer.serialize_str(self.to_string().as_ref()),
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extension_errors_keep_their_json_shape() {
        let error = Error::Extension(crate::ExtensionError::from_value(serde_json::json!({
            "code": "POLICY_REJECTED",
            "host": "example.test"
        })));

        assert_eq!(
            serde_json::to_value(error).unwrap(),
            serde_json::json!({
                "code": "POLICY_REJECTED",
                "host": "example.test"
            })
        );
    }

    #[test]
    fn existing_errors_remain_strings() {
        let error = Error::RequestCanceled;
        assert_eq!(
            serde_json::to_value(error).unwrap(),
            serde_json::Value::String("Request canceled".into())
        );
    }
}
