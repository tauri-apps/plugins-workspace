// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("current executable path has no parent")]
    CurrentExeHasNoParent,
    #[error("unknown program {0}")]
    UnknownProgramName(String),
    #[error(transparent)]
    Scope(#[from] crate::scope::Error),
    /// Sidecar not allowed by the configuration.
    #[error("sidecar not configured under `tauri.conf.json > tauri > bundle > externalBin`: {0}")]
    SidecarNotAllowed(PathBuf),
    /// Program not allowed by the scope.
    #[error("program not allowed on the configured shell scope: {0}")]
    ProgramNotAllowed(PathBuf),
    #[error("unknown encoding {0}")]
    UnknownEncoding(String),
    /// JSON error.
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    /// Utf8 error.
    #[error(transparent)]
    Utf8(#[from] std::string::FromUtf8Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
