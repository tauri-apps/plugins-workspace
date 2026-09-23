// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    /// Whether or not paths that contain components that start with a `.`
    /// will require that `.` appears literally in the pattern; `*`, `?`, `**`,
    /// or `[...]` will not match. This is useful because such files are
    /// conventionally considered hidden on Unix systems and it might be
    /// desirable to skip them when listing files.
    ///
    /// Defaults to `true` on Unix systems and `false` on Windows
    // dotfiles are not supposed to be exposed by default on unix
    pub require_literal_leading_dot: Option<bool>,
    /// Whether the paths dropped onto a window (drag and drop) are added to the runtime fs scope,
    /// directories recursively, which gives every webview with an fs permission access to them.
    ///
    /// Defaults to `true`.
    pub scope_dropped_paths: Option<bool>,
}
