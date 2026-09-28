// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::{Serialize, Serializer};

/// The error type returned by the opener plugin's commands and APIs.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// Forwarded from a failed call into the mobile plugin runtime.
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    /// Forwarded from a [`tauri::Error`], e.g. when resolving a scoped path fails.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    /// Forwarded from an [`std::io::Error`] raised while opening or revealing a path.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Forwarded from a [`serde_json::Error`] raised while (de)serializing a mobile plugin payload.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// The program passed as `with` is not a program name known by the underlying opener.
    #[error("unknown program {0}")]
    UnknownProgramName(String),
    /// The path is not allowed by the opener scope, optionally together with the program it was requested to be opened with.
    #[error("Not allowed to open path {}{}", .path, .with.as_ref().map(|w| format!(" with {w}")).unwrap_or_default())]
    ForbiddenPath {
        /// The path that was rejected by the scope.
        path: String,
        /// The program the path was requested to be opened with, if any.
        with: Option<String>,
    },
    /// The URL is not allowed by the opener scope, optionally together with the program it was requested to be opened with.
    #[error("Not allowed to open url {}{}", .url, .with.as_ref().map(|w| format!(" with {w}")).unwrap_or_default())]
    ForbiddenUrl {
        /// The URL that was rejected by the scope.
        url: String,
        /// The program the URL was requested to be opened with, if any.
        with: Option<String>,
    },
    /// The requested API is not supported on the current platform, e.g. [`crate::reveal_item_in_dir`] on Android and iOS.
    #[error("API not supported on the current platform")]
    UnsupportedPlatform,
    /// Forwarded from a Win32 API call, see [`windows::core::Error`].
    #[error(transparent)]
    #[cfg(windows)]
    Win32Error(#[from] windows::core::Error),
    /// The given path has no parent directory, so it cannot be revealed in its containing folder.
    #[error("Path doesn't have a parent: {0}")]
    NoParent(PathBuf),
    // TODO: Add the underlying io::Error to this variant
    /// Failed to convert the path to a Windows `ITEMIDLIST` while preparing it to be revealed in the file explorer.
    #[cfg(windows)]
    #[error("Failed to convert path '{0}' to ITEMIDLIST")]
    FailedToConvertPathToItemIdList(PathBuf),
    /// Failed to convert the path to a `file://` URL, which is required to reveal it via D-Bus on Linux and BSD.
    #[error("Failed to convert path to file:// url")]
    FailedToConvertPathToFileUrl,
    /// Forwarded from a [`zbus::Error`] raised while talking to the file manager or the desktop portal over D-Bus.
    #[error(transparent)]
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    Zbus(#[from] zbus::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
