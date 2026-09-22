// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

/// Alias for the result type returned by the geolocation APIs.
pub type Result<T> = std::result::Result<T, Error>;

// TODO: Improve Error handling (different typed errors instead of one (stringified) PluginInvokeError for all mobile errors)

/// Errors that can occur while interacting with the geolocation APIs.
#[derive(Debug, thiserror::Error)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
pub enum Error {
    /// Forwards an error returned by the underlying Android or iOS plugin invocation.
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(
        #[cfg_attr(feature = "specta", specta(skip))]
        #[from]
        tauri::plugin::mobile::PluginInvokeError,
    ),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
