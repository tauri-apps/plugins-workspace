// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Interface with SQL databases through [sqlx](https://github.com/launchbadge/sqlx).
//!
//! ## Cargo features
//!
//! No database driver is enabled by default. Enable at least one driver;
//! multiple drivers may be enabled together.
//!
//! - **sqlite**: Adds support for SQLite.
//! - **mysql**: Adds support for MySQL.
//! - **postgres**: Adds support for PostgreSQL.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

mod commands;
mod decode;
mod error;
mod wrapper;

pub use error::Error;
pub use wrapper::DbPool;

use futures_core::future::BoxFuture;
use serde::{Deserialize, Serialize};
use sqlx::{
    error::BoxDynError,
    migrate::{Migration as SqlxMigration, MigrationSource, MigrationType, Migrator},
};
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, RunEvent, Runtime,
};
use tokio::sync::{Mutex, RwLock};

use std::collections::HashMap;

/// The connection pools the plugin currently holds, keyed by the connection
/// string the database was loaded with.
///
/// It is managed as Tauri state, so Rust code can reach the pools with
/// [`tauri::Manager::state`] and run its own queries.
#[derive(Default)]
pub struct DbInstances(pub RwLock<HashMap<String, DbPool>>);

#[derive(Serialize)]
#[serde(untagged)]
pub(crate) enum LastInsertId {
    #[cfg(feature = "sqlite")]
    Sqlite(i64),
    #[cfg(feature = "mysql")]
    MySql(u64),
    #[cfg(feature = "postgres")]
    Postgres(()),
    #[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "postgres")))]
    None,
}

struct Migrations(Mutex<HashMap<String, MigrationList>>);

/// The `plugins > sql` section of the Tauri configuration file.
#[derive(Default, Clone, Deserialize)]
pub struct PluginConfig {
    /// Connection strings of the databases to connect to when the application
    /// starts. Empty by default.
    #[serde(default)]
    preload: Vec<String>,
}

/// The direction of a [`Migration`].
#[derive(Debug)]
pub enum MigrationKind {
    /// Moves the schema forward. Only migrations of this kind are executed by the plugin.
    Up,
    /// Reverts an [`Up`](Self::Up) migration. Migrations of this kind are currently
    /// never executed by the plugin.
    Down,
}

impl From<MigrationKind> for MigrationType {
    fn from(kind: MigrationKind) -> Self {
        match kind {
            MigrationKind::Up => Self::ReversibleUp,
            MigrationKind::Down => Self::ReversibleDown,
        }
    }
}

/// A migration definition.
///
/// Migrations are attached to a database with [`Builder::add_migrations`] and run
/// the first time that database is connected to - on startup for the databases
/// listed in the `preload` configuration, otherwise when the frontend loads it.
/// Only [`MigrationKind::Up`] migrations are executed, in ascending
/// [`version`](Self::version) order, and sqlx keeps track of the versions that
/// already ran so each one is applied at most once per database.
#[derive(Debug)]
pub struct Migration {
    /// The version of this migration. Determines the order in which migrations
    /// run and identifies the migration in the database.
    pub version: i64,
    /// A human readable description of what the migration does.
    pub description: &'static str,
    /// The SQL executed when the migration runs.
    pub sql: &'static str,
    /// Whether this migration moves the schema forward or reverts it.
    pub kind: MigrationKind,
}

#[derive(Debug)]
struct MigrationList(Vec<Migration>);

impl MigrationSource<'static> for MigrationList {
    fn resolve(self) -> BoxFuture<'static, std::result::Result<Vec<SqlxMigration>, BoxDynError>> {
        Box::pin(async move {
            let mut migrations = Vec::new();
            for migration in self.0 {
                if matches!(migration.kind, MigrationKind::Up) {
                    migrations.push(SqlxMigration::new(
                        migration.version,
                        migration.description.into(),
                        migration.kind.into(),
                        migration.sql.into(),
                        false,
                    ));
                }
            }
            Ok(migrations)
        })
    }
}

/// Allows blocking on async code without creating a nested runtime.
fn run_async_command<F: std::future::Future>(cmd: F) -> F::Output {
    if tokio::runtime::Handle::try_current().is_ok() {
        tokio::task::block_in_place(|| tokio::runtime::Handle::current().block_on(cmd))
    } else {
        tauri::async_runtime::block_on(cmd)
    }
}

/// Tauri SQL plugin builder.
#[derive(Default)]
pub struct Builder {
    migrations: Option<HashMap<String, MigrationList>>,
}

impl Builder {
    /// Creates a new builder with no migrations registered.
    ///
    /// Prints a message to stderr when none of the `sqlite`, `mysql` and
    /// `postgres` Cargo features is enabled, since no database can be
    /// connected to in that case.
    pub fn new() -> Self {
        #[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "postgres")))]
        eprintln!("No sql driver enabled. Please set at least one of the \"sqlite\", \"mysql\", \"postgres\" feature flags.");

        Self::default()
    }

    /// Add migrations to a database.
    #[must_use]
    pub fn add_migrations(mut self, db_url: &str, migrations: Vec<Migration>) -> Self {
        self.migrations
            .get_or_insert(Default::default())
            .insert(db_url.to_string(), MigrationList(migrations));
        self
    }

    /// Builds the SQL plugin.
    ///
    /// On setup the plugin connects to every database listed in the `preload`
    /// array of its [configuration](PluginConfig), running the migrations
    /// registered for them, and it closes all connection pools when the
    /// application exits.
    ///
    /// # Examples
    ///
    /// ```
    /// use tauri_plugin_sql::{Builder, Migration, MigrationKind};
    ///
    /// fn sql_plugin<R: tauri::Runtime>(
    /// ) -> tauri::plugin::TauriPlugin<R, Option<tauri_plugin_sql::PluginConfig>> {
    ///     Builder::new()
    ///         .add_migrations(
    ///             "sqlite:mydatabase.db",
    ///             vec![Migration {
    ///                 version: 1,
    ///                 description: "create todos table",
    ///                 sql: "CREATE TABLE todos (id INTEGER PRIMARY KEY, title TEXT);",
    ///                 kind: MigrationKind::Up,
    ///             }],
    ///         )
    ///         .build()
    /// }
    /// ```
    pub fn build<R: Runtime>(mut self) -> TauriPlugin<R, Option<PluginConfig>> {
        PluginBuilder::<R, Option<PluginConfig>>::new("sql")
            .invoke_handler(tauri::generate_handler![
                commands::load,
                commands::execute,
                commands::select,
                commands::close
            ])
            .setup(|app, api| {
                let config = api.config().clone().unwrap_or_default();

                run_async_command(async move {
                    let instances = DbInstances::default();
                    let mut lock = instances.0.write().await;

                    for db in config.preload {
                        let pool = DbPool::connect(&db, app).await?;

                        if let Some(migrations) =
                            self.migrations.as_mut().and_then(|mm| mm.remove(&db))
                        {
                            let migrator = Migrator::new(migrations).await?;
                            pool.migrate(&migrator).await?;
                        }

                        lock.insert(db, pool);
                    }
                    drop(lock);

                    app.manage(instances);
                    app.manage(Migrations(Mutex::new(
                        self.migrations.take().unwrap_or_default(),
                    )));

                    Ok(())
                })
            })
            .on_event(|app, event| {
                if let RunEvent::Exit = event {
                    run_async_command(async move {
                        let instances = &*app.state::<DbInstances>();
                        let instances = instances.0.read().await;
                        for value in instances.values() {
                            value.close().await;
                        }
                    });
                }
            })
            .build()
    }
}
