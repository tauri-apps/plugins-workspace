// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};
use std::path::PathBuf;

pub type Result<T> = std::result::Result<T, Error>;

/// The error types.
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    /// Mobile plugin handled is not initialized, Probably [`StoreBuilder::mobile_plugin_handle`] was not called.
    #[cfg(mobile)]
    #[error("Mobile plugin handled is not initialized, Perhaps you forgot to call StoreBuilder::mobile_plugin_handle")]
    MobilePluginHandleUnInitialized,
    #[error("Failed to serialize store. {0}")]
    Serialize(Box<dyn std::error::Error + Send + Sync>),
    #[error("Failed to deserialize store. {0}")]
    Deserialize(Box<dyn std::error::Error + Send + Sync>),
    /// JSON error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// IO error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// Store not found
    #[error("Store \"{0}\" not found")]
    NotFound(PathBuf),
    /// Some Tauri API failed
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
