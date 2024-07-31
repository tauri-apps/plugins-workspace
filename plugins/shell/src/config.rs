// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;

/// Configuration for the shell plugin.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    /// Open URL with the user's default application.
    #[serde(default)]
    pub open: ShellAllowlistOpen,
}

/// Defines the `shell > open` api scope.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowlistOpen {
    /// If the shell open API should be enabled.
    ///
    /// If enabled, the default validation regex (`^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`) is used.
    Flag(bool),

    /// Enable the shell open API, with a custom regex that the opened path must match against.
    ///
    /// The regex string is automatically surrounded by `^...$` to match the full string.
    /// For example the `https?://\w+` regex would be registered as `^https?://\w+$`.
    ///
    /// If using a custom regex to support a non-http(s) schema, care should be used to prevent values
    /// that allow flag-like strings to pass validation. e.g. `--enable-debugging`, `-i`, `/R`.
    Validate(String),
}

impl Default for ShellAllowlistOpen {
    fn default() -> Self {
        Self::Flag(false)
    }
}
