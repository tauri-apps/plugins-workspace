// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, Serializer};

/// Errors that can happen while connecting to a database, running migrations
/// or executing a query.
///
/// Serializes to its [`std::fmt::Display`] representation, which is what the
/// frontend receives when a command fails.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error reported by [`sqlx`], such as a failed connection or a query the database rejected.
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    /// A migration registered with [`crate::Builder::add_migrations`] could not be resolved or applied.
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    /// The connection string is missing its `scheme:` prefix, or the scheme does not
    /// match any of the enabled database drivers. Contains the offending connection string.
    #[error("invalid connection url: {0}")]
    InvalidDbUrl(String),
    /// The requested database has not been connected to with the `load` command
    /// and is not listed in the plugin's `preload` configuration.
    /// Contains the connection string of the database.
    #[error("database {0} not loaded")]
    DatabaseNotLoaded(String),
    /// A value selected from the database has a SQL type that cannot be converted
    /// to JSON. Contains the name of that SQL type.
    #[error("unsupported datatype: {0}")]
    UnsupportedDatatype(String),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
