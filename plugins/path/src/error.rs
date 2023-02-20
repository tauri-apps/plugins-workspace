// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("path does not have a parent")]
    NoParent,
    #[error("path does not have an extension")]
    NoExtension,
    #[error("path does not have a basename")]
    NoBasename,
    #[error("failed to read current dir: {0}")]
    CurrentDir(std::io::Error),
    #[cfg(desktop)]
    #[error("unknown path")]
    UnknownPath,
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
