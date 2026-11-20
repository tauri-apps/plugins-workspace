// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(untagged)]
#[derive(Default)]
pub enum Application {
    #[default]
    Default,
    Enable(bool),
    App(String),
}

#[derive(Deserialize)]
#[serde(untagged, rename_all = "camelCase")]
pub(crate) enum EntryRaw {
    Url {
        url: String,
        #[serde(default)]
        app: Application,
    },
    Path {
        path: PathBuf,
        #[serde(default)]
        app: Application,
    },
}
