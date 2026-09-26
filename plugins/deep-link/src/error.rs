// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, ser::Serializer};

/// Alias for a [`Result`](std::result::Result) with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// The error type for this plugin.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// The requested operation (usually registering or unregistering a protocol scheme at
    /// runtime) is not supported on the current platform.
    #[error("unsupported platform")]
    UnsupportedPlatform,
    /// Transparent wrapper around an [`std::io::Error`].
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Transparent wrapper around a [`tauri::Error`].
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    /// Transparent wrapper around a [`windows_result::Error`]. Only used on Windows.
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    Windows(#[from] windows_result::Error),
    /// Transparent wrapper around an [`ini::Error`], returned when reading or writing the
    /// `.desktop` file used to register a protocol scheme. Only used on Linux.
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    Ini(#[from] ini::Error),
    /// Transparent wrapper around an [`ini::ParseError`], returned when parsing the
    /// `.desktop` file used to register a protocol scheme. Only used on Linux.
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    ParseIni(#[from] ini::ParseError),
    /// Failed to run an OS command such as `xdg-mime` or `update-desktop-database`.
    #[cfg(target_os = "linux")]
    #[error("Failed to run OS command `{0}`: {1}")]
    Execute(&'static str, #[source] std::io::Error),
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
