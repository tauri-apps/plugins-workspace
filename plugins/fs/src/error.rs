// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::{Serialize, Serializer};

/// Errors that can happen while using the file system plugin.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// JSON serialization or deserialization error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Error from the Tauri APIs, usually raised while resolving a path or a scope entry.
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    /// Error from the underlying file system operation.
    #[error(transparent)]
    Io(#[from] std::io::Error),
    /// The path is denied by the plugin scope or is not allowed by it.
    #[error("forbidden path: {0}")]
    PathForbidden(PathBuf),
    /// Invalid glob pattern.
    #[error("invalid glob pattern: {0}")]
    GlobPattern(#[from] glob::PatternError),
    /// Watcher error.
    #[cfg(feature = "watch")]
    #[error(transparent)]
    Watch(#[from] notify::Error),
    /// Error invoking the Android plugin implementation.
    #[cfg(target_os = "android")]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    /// The URL cannot be converted to a file system path.
    #[error("URL is not a valid path")]
    InvalidPathUrl,
    /// The path is not safe to use, for instance because it traverses parent directories.
    #[error("Unsafe PathBuf: {0}")]
    UnsafePathBuf(&'static str),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
