// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use tauri::utils::config::FsScope;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub scope: FsScope,
}
