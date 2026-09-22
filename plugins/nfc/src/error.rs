// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

/// Alias for a [`std::result::Result`] with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the NFC plugin.
///
/// Serializes to the error message string, so it can be returned directly from a command.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An I/O error happened.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The call to the Android or iOS plugin implementation failed,
    /// either because the arguments could not be serialized, the response could not be
    /// deserialized or the native side rejected the call (e.g. NFC is unavailable
    /// or the tag could not be read or written).
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
