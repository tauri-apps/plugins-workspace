// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unsupported platform")]
    UnsupportedPlatform,
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[cfg(target_os = "windows")]
    #[error(transparent)]
    Windows(#[from] windows_result::Error),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    Ini(#[from] ini::Error),
    #[cfg(target_os = "linux")]
    #[error(transparent)]
    ParseIni(#[from] ini::ParseError),
    #[cfg(mobile)]
    #[error(transparent)]
    PluginInvoke(#[from] tauri::plugin::mobile::PluginInvokeError),
}

// TODO(v3): change this into an error in v3,
// see <https://github.com/tauri-apps/plugins-workspace/pull/2970#issuecomment-3244660138>.
#[inline]
#[cfg(target_os = "linux")]
pub(crate) fn inspect_command_error<'a>(command: &'a str) -> impl Fn(&std::io::Error) + 'a {
    move |e| {
        tracing::error!("Failed to run OS command `{command}`: {e}");
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
