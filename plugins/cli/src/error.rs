// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};

/// Errors that can be returned from the CLI plugin's commands.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Parsing the process arguments against the CLI definition in `tauri.conf.json` failed,
    /// e.g. because a required argument is missing or an unknown flag was passed.
    #[error("failed to parse arguments: {0}")]
    ParseCli(#[from] clap::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

/// Alias for a [`std::result::Result`] with the error type [`Error`].
pub type Result<T> = std::result::Result<T, Error>;
