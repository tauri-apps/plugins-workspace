// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug)]
pub struct Entry {
    pub path: Option<PathBuf>,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum EntryRaw {
    Value(PathBuf),
    Object { path: PathBuf },
}
