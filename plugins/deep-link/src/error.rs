// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

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
    /// `.desktop` file used to register a protocol scheme. Only used on Linux and FreeBSD.
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    #[error(transparent)]
    Ini(#[from] ini::Error),
    /// Transparent wrapper around an [`ini::ParseError`], returned when parsing the
    /// `.desktop` file used to register a protocol scheme. Only used on Linux and FreeBSD.
    #[cfg(any(target_os = "linux", target_os = "freebsd"))]
    #[error(transparent)]
    ParseIni(#[from] ini::ParseError),
    /// Transparent wrapper around a [`tauri::plugin::mobile::PluginInvokeError`], returned
    /// when the underlying mobile plugin invocation fails. Only used on mobile.
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
}

// TODO(v3): change this into an error in v3,
// see <https://github.com/tauri-apps/plugins-workspace/pull/2970#issuecomment-3244660138>.
#[inline]
#[cfg(any(target_os = "linux", target_os = "freebsd"))]
pub(crate) fn inspect_command_error<'a>(command: &'a str) -> impl Fn(&std::io::Error) + 'a {
    move |e| {
        tracing::error!("Failed to run OS command `{command}`: {e}");
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
