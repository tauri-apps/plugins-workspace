// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

/// Alias for a [`std::result::Result`] with the error type set to [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the notification plugin.
///
/// The error is serialized to its [`Display`](std::fmt::Display) string when it crosses the IPC boundary.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O operation failed, e.g. resolving the path of the running executable on Windows.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Failed to run a command on the mobile plugin implementation (Kotlin on Android, Swift on iOS).
    ///
    /// Only available on mobile.
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
