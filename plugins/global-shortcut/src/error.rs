// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};

/// Errors that can happen while registering, unregistering or parsing global shortcuts.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    /// An error returned by the underlying `global_hotkey` crate, for example when the OS
    /// refuses to register or unregister a shortcut, or when a shortcut string fails to parse.
    #[error("{0}")]
    GlobalHotkey(String),
    /// Failed to receive the result of an operation dispatched to the main thread because the
    /// sending end of the channel was dropped before it could reply.
    #[error(transparent)]
    RecvError(#[from] std::sync::mpsc::RecvError),
    /// An error returned by the Tauri runtime, for example when dispatching a closure to run on
    /// the main thread fails.
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
