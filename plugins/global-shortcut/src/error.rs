// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("{0}")]
    GlobalHotkey(String),
    #[error(transparent)]
    RecvError(#[from] std::sync::mpsc::RecvError),
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

impl From<global_hotkey::Error> for Error {
    fn from(value: global_hotkey::Error) -> Self {
        Self::GlobalHotkey(value.to_string())
    }
}

impl From<global_hotkey::hotkey::HotKeyParseError> for Error {
    fn from(value: global_hotkey::hotkey::HotKeyParseError) -> Self {
        Self::GlobalHotkey(value.to_string())
    }
}
