// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, ser::Serializer};

/// Alias for a [`Result`](std::result::Result) with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The error types returned by this plugin.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O error occurred.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The invocation of the underlying Android or iOS plugin failed, for example because
    /// biometric authentication was unavailable, was not enrolled, failed, or was canceled by
    /// the user.
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    /// The authentication prompt failed or was canceled by the user or the system.
    #[cfg(desktop)]
    #[error("Biometric authentication failed: {message}")]
    AuthenticationFailed {
        /// A human-readable description of the failure, as reported by the operating system.
        message: String,
        /// The error code, using the same identifiers as [`Status::error_code`](crate::Status::error_code).
        code: Option<String>,
    },
    /// Biometric authentication cannot be used on this device.
    #[cfg(desktop)]
    #[error("Biometric unavailable: {message}")]
    Unavailable {
        /// A human-readable reason why biometric authentication is unavailable.
        message: String,
        /// The error code, using the same identifiers as [`Status::error_code`](crate::Status::error_code).
        code: Option<String>,
    },
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            // Same `{ message, code }` shape the Android and iOS plugins reject with.
            #[cfg(desktop)]
            Self::AuthenticationFailed { message, code } | Self::Unavailable { message, code } => {
                #[derive(Serialize)]
                struct Payload<'a> {
                    message: &'a str,
                    #[serde(skip_serializing_if = "Option::is_none")]
                    code: Option<&'a str>,
                }
                Payload {
                    message,
                    code: code.as_deref(),
                }
                .serialize(serializer)
            }
            _ => serializer.serialize_str(self.to_string().as_ref()),
        }
    }
}

#[cfg(all(test, desktop))]
mod tests {
    use super::Error;

    #[test]
    fn desktop_errors_serialize_like_mobile_rejections() {
        let err = Error::AuthenticationFailed {
            message: "Canceled by user.".into(),
            code: Some("userCancel".into()),
        };
        assert_eq!(
            serde_json::to_value(&err).unwrap(),
            serde_json::json!({ "message": "Canceled by user.", "code": "userCancel" })
        );

        let err = Error::Unavailable {
            message: "No biometry".into(),
            code: None,
        };
        assert_eq!(
            serde_json::to_value(&err).unwrap(),
            serde_json::json!({ "message": "No biometry" })
        );
    }
}
