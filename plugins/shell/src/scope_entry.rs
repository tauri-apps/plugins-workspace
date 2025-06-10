// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{de::Error as DeError, Deserialize, Deserializer};

use std::path::PathBuf;

/// A command allowed to be executed by the webview API.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct Entry {
    pub(crate) name: String,
    pub(crate) command: PathBuf,
    pub(crate) args: ShellAllowedArgs,
    pub(crate) sidecar: bool,
}

#[derive(Deserialize)]
pub(crate) struct EntryRaw {
    pub(crate) name: String,
    #[serde(rename = "cmd")]
    pub(crate) command: Option<PathBuf>,
    #[serde(default)]
    pub(crate) args: ShellAllowedArgs,
    #[serde(default)]
    pub(crate) sidecar: bool,
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let config = EntryRaw::deserialize(deserializer)?;

        if !config.sidecar && config.command.is_none() {
            return Err(DeError::custom(
                "The shell scope `command` value is required.",
            ));
        }

        Ok(Entry {
            name: config.name,
            command: config.command.unwrap_or_default(),
            args: config.args,
            sidecar: config.sidecar,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowedArgs {
    Flag(bool),
    List(Vec<ShellAllowedArg>),
}

impl Default for ShellAllowedArgs {
    fn default() -> Self {
        Self::Flag(false)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowedArg {
    Fixed(String),
    Var {
        validator: String,
        #[serde(default)]
        raw: bool,
    },
}
