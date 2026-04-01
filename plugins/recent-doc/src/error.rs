// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
#[allow(unused)]
pub enum Error {
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    // Transform windows error into our error type
    #[error(transparent)]
    #[cfg(target_os = "windows")]
    WindowsErr(#[from] windows::core::Error),
    #[error(transparent)]
    #[cfg(target_os = "windows")]
    Utf16(#[from] std::string::FromUtf16Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
