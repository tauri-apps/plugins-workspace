// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, ser::Serializer};

/// Alias for `Result<T, Error>` used throughout this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur while showing or interacting with a dialog.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An error forwarded from the Tauri core.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    /// An I/O error, for example while resolving a picked path.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Forwarding a request to, or receiving a response from, the mobile plugin failed.
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    /// The folder picker was requested through the `open` command, but folder picking is not implemented on mobile.
    #[cfg(mobile)]
    #[error("Folder picker is not implemented on mobile")]
    FolderPickerNotImplemented,
    /// An error forwarded from the `fs` plugin, returned when granting filesystem scope to a picked path fails.
    #[error(transparent)]
    Fs(#[from] tauri_plugin_fs::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

// Propagate contextual failures instead of treating them as cancellation.
#[cfg(any(mobile, test))]
fn is_contextual_error(code: Option<&str>) -> bool {
    matches!(code, Some("ORIGIN_UNAVAILABLE" | "RESULT_PENDING"))
}

// Preserve the legacy fallback for other failures: None for files, Cancel for messages.
#[cfg(any(mobile, test))]
pub(crate) fn or_previous_result<T, E>(
    result: std::result::Result<T, E>,
    previous: T,
    error_code: impl FnOnce(&E) -> Option<&str>,
) -> std::result::Result<T, E> {
    match result {
        Err(error) if !is_contextual_error(error_code(&error)) => Ok(previous),
        result => result,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contextual_errors_propagate_and_other_failures_keep_the_previous_result() {
        for (code, contextual) in [
            (Some("ORIGIN_UNAVAILABLE"), true),
            (Some("RESULT_PENDING"), true),
            (Some("OTHER_ERROR"), false),
            (None, false),
        ] {
            assert_eq!(is_contextual_error(code), contextual);
            assert_eq!(
                or_previous_result(Err(code), 7, |code| *code),
                if contextual { Err(code) } else { Ok(7) }
            );
        }
        assert_eq!(
            or_previous_result(Ok::<_, Option<&str>>(42), 7, |_| panic!(
                "success has no error"
            )),
            Ok(42)
        );
    }
}
