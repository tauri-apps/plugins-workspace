// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

/// Alias for `Result<T, Error>` used throughout this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur while interacting with the system clipboard.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Forwarding a request to, or receiving a response from, the mobile plugin failed.
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    /// The underlying clipboard operation failed, or the operation is not supported on this platform.
    #[error("{0}")]
    Clipboard(String),
    /// An error forwarded from the Tauri core.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg(desktop)]
impl From<arboard::Error> for Error {
    fn from(error: arboard::Error) -> Self {
        Self::Clipboard(error.to_string())
    }
}
