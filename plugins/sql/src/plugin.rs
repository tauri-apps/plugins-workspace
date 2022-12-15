// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
use serde::{ser::Serializer, Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[cfg(feature = "mssql")]
use sqlx::mssql::MssqlArguments;
#[cfg(feature = "mysql")]
type SqlArguments<'a> = sqlx::mysql::MySqlArguments;
#[cfg(feature = "postgres")]
type SqlArguments<'a> = sqlx::postgres::PgArguments;
#[cfg(feature = "sqlite")]
type SqlArguments<'a> = sqlx::sqlite::SqliteArguments<'a>;

#[cfg(not(feature = "mssql"))]
use futures::future::BoxFuture;
#[cfg(not(feature = "mssql"))]
use sqlx::{
    error::BoxDynError,
    migrate::{
        MigrateDatabase, Migration as SqlxMigration, MigrationSource, MigrationType, Migrator,
    },
};

use sqlx::{query::Query, Column, Pool, Row};
use std::collections::HashMap;
use tauri::{
    command,
    plugin::{Plugin, Result as PluginResult},
    AppHandle, Invoke, Manager, RunEvent, Runtime, State,
};
use tokio::sync::Mutex;
use tracing::info;

#[cfg(feature = "sqlite")]
use std::{fs::create_dir_all, path::PathBuf};

#[cfg(feature = "sqlite")]
type Db = sqlx::sqlite::Sqlite;
#[cfg(feature = "mysql")]
type Db = sqlx::mysql::MySql;
#[cfg(feature = "postgres")]
type Db = sqlx::postgres::Postgres;
#[cfg(feature = "mssql")]
type Db = sqlx::mssql::Mssql;

#[cfg(feature = "sqlite")]
type LastInsertId = i64;
#[cfg(not(feature = "sqlite"))]
type LastInsertId = u64;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sql(#[from] sqlx::Error),
    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("database {0} not loaded")]
    DatabaseNotLoaded(String),
    #[error("Could not decode the numeric column {0} into a type for {1} database.")]
    NumericDecoding(String, String),
    #[error("Sqlite doesn't have a native Boolean type but represents boolean values as an integer value of 0 or 1, however we received a value of {0} for the column {1}")]
    BooleanDecoding(String, String),
    #[error("Non-string based query is not allowed with this database")]
    NonStringQuery,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

use crate::deserialize::deserialize_col;

type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "sqlite")]
/// Resolves the App's **file path** from the `AppHandle`
/// context object
fn app_path<R: Runtime>(app: &AppHandle<R>) -> PathBuf {
    #[allow(deprecated)] // FIXME: Change to non-deprecated function in Tauri v2
    app.path_resolver()
        .app_dir()
        .expect("No App path was found!")
}

#[cfg(feature = "sqlite")]
/// Maps the user supplied DB connection string to a connection string
/// with a fully qualified file path to the App's designed "app_path"
fn path_mapper(mut app_path: PathBuf, connection_string: &str) -> String {
    app_path.push(
        connection_string
            .split_once(':')
            .expect("Couldn't parse the connection string for DB!")
            .1,
    );

    format!(
        "sqlite:{}",
        app_path
            .to_str()
            .expect("Problem creating fully qualified path to Database file!")
    )
}

#[derive(Default)]
struct DbInstances(Mutex<HashMap<String, Pool<Db>>>);

#[cfg(not(feature = "mssql"))]
struct Migrations(Mutex<HashMap<String, MigrationList>>);
#[cfg(feature = "mssql")]
struct Migrations();

#[derive(Default, Deserialize)]
struct PluginConfig {
    #[serde(default)]
    preload: Vec<String>,
}

#[derive(Debug)]
#[cfg(not(feature = "mssql"))]
pub enum MigrationKind {
    Up,
    Down,
}

#[cfg(not(feature = "mssql"))]
impl From<MigrationKind> for MigrationType {
    fn from(kind: MigrationKind) -> Self {
        match kind {
            MigrationKind::Up => Self::ReversibleUp,
            MigrationKind::Down => Self::ReversibleDown,
        }
    }
}

/// A migration definition.
#[derive(Debug)]
#[cfg(not(feature = "mssql"))]
pub struct Migration {
    pub version: i64,
    pub description: &'static str,
    pub sql: &'static str,
    pub kind: MigrationKind,
}

#[derive(Debug)]
#[cfg(not(feature = "mssql"))]
struct MigrationList(Vec<Migration>);

#[cfg(not(feature = "mssql"))]
impl MigrationSource<'static> for MigrationList {
    #[tracing::instrument]
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
                    ));
                }
            }
            Ok(migrations)
        })
    }
}

#[command]
async fn load<R: Runtime>(
    #[allow(unused_variables)] app: AppHandle<R>,
    db_instances: State<'_, DbInstances>,
    migrations: State<'_, Migrations>,
    db: String,
) -> Result<String> {
    #[cfg(feature = "sqlite")]
    let fqdb = path_mapper(app_path(&app), &db);
    #[cfg(not(feature = "sqlite"))]
    let fqdb = db.clone();

    #[cfg(feature = "sqlite")]
    create_dir_all(app_path(&app)).expect("Problem creating App directory!");

    // currently sqlx can not create a mssql database
    #[cfg(not(feature = "mssql"))]
    if !Db::database_exists(&fqdb).await.unwrap_or(false) {
        Db::create_database(&fqdb).await?;
    }

    let pool = Pool::connect(&fqdb).await?;

    #[cfg(not(feature = "mssql"))]
    if let Some(migrations) = migrations.0.lock().await.remove(&db) {
        let migrator = Migrator::new(migrations).await?;
        migrator.run(&pool).await?;
    }

    db_instances.0.lock().await.insert(db.clone(), pool);
    info!("Database pool \"{}\" has been loaded", db.clone());

    Ok(db)
}

/// Allows the database connection(s) to be closed; if no database
/// name is passed in then _all_ database connection pools will be
/// shut down.
#[command]
async fn close(db_instances: State<'_, DbInstances>, db: Option<String>) -> Result<bool> {
    let mut instances = db_instances.0.lock().await;

    let pools = if let Some(db) = db {
        vec![db]
    } else {
        instances.keys().cloned().collect()
    };

    info!(
        "{} databases closed explicitly in close() call.",
        pools.len().to_string()
    );

    for pool in pools {
        let db = instances
            .get_mut(&pool) //
            .ok_or(Error::DatabaseNotLoaded(pool))?;
        db.close().await;
    }

    Ok(true)
}

/// Execute a command against the database
#[command]
async fn execute(
    db_instances: State<'_, DbInstances>,
    db: String,
    sql: String,
    values: Vec<JsonValue>,
) -> Result<(u64, LastInsertId)> {
    let mut instances = db_instances.0.lock().await;
    let db = instances
        .get_mut(&db) //
        .ok_or(Error::DatabaseNotLoaded(db))?;
    let mut query = sqlx::query(&sql);
    for value in values {
        query = bind_query(query, value)?;
    }
    let result = query.execute(&*db).await?;
    info!("successful database execute() command: {}", &sql);

    #[cfg(feature = "sqlite")]
    let r = Ok((result.rows_affected(), result.last_insert_rowid()));
    #[cfg(feature = "mysql")]
    let r = Ok((result.rows_affected(), result.last_insert_id()));
    #[cfg(feature = "postgres")]
    let r = Ok((result.rows_affected(), 0));
    #[cfg(feature = "mssql")]
    let r = Ok((result.rows_affected(), 0));

    r
}

#[cfg(feature = "mssql")]
fn bind_query(
    mut query: Query<Db, MssqlArguments>,
    value: JsonValue,
) -> Result<Query<Db, MssqlArguments>> {
    if value.is_string() {
        query = query.bind(value.as_str().unwrap().to_owned());
        Ok(query)
    } else {
        Err(Error::NonStringQuery)
    }
}
#[cfg(not(feature = "mssql"))]
fn bind_query<'a>(
    mut query: Query<'a, Db, SqlArguments<'a>>,
    value: JsonValue,
) -> Result<Query<'a, Db, SqlArguments<'a>>> {
    if value.is_string() {
        query = query.bind(value.as_str().unwrap().to_owned());
        Ok(query)
    } else {
        query = query.bind(value);
        Ok(query)
    }
}

#[command]
async fn select(
    db_instances: State<'_, DbInstances>,
    db: String,
    sql: String,
    values: Vec<JsonValue>,
) -> Result<Vec<HashMap<String, JsonValue>>> {
    let mut instances = db_instances.0.lock().await;
    let db = instances.get_mut(&db).ok_or(Error::DatabaseNotLoaded(db))?;
    let mut query = sqlx::query(&sql);

    for value in values {
        query = bind_query(query, value)?;
    }

    let rows = query.fetch_all(&*db).await?;
    let mut values = Vec::new();
    for row in rows {
        let mut value = HashMap::default();
        for (i, column) in row.columns().iter().enumerate() {
            let v = deserialize_col(&row, column, &i)?;
            value.insert(column.name().to_string(), v);
        }
        values.push(value);
    }

    info!("successful select() query: {}", sql);

    Ok(values)
}

/// Tauri SQL plugin.
pub struct TauriSql<R: Runtime> {
    #[cfg(not(feature = "mssql"))]
    migrations: Option<HashMap<String, MigrationList>>,
    invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> Default for TauriSql<R> {
    fn default() -> Self {
        Self {
            #[cfg(not(feature = "mssql"))]
            migrations: Some(Default::default()),
            invoke_handler: Box::new(tauri::generate_handler![load, execute, select, close]),
        }
    }
}

impl<R: Runtime> TauriSql<R> {
    /// Add migrations to a database.
    #[must_use]
    #[cfg(not(feature = "mssql"))]
    pub fn add_migrations(mut self, db_url: &str, migrations: Vec<Migration>) -> Self {
        self.migrations
            .as_mut()
            .unwrap()
            .insert(db_url.to_string(), MigrationList(migrations));

        info!("migrations on database have finished");

        self
    }
}

impl<R: Runtime> Plugin<R> for TauriSql<R> {
    fn name(&self) -> &'static str {
        "sql"
    }

    fn initialize(
        &mut self,
        app: &AppHandle<R>,
        user_config: serde_json::Value,
    ) -> PluginResult<()> {
        tauri::async_runtime::block_on(async move {
            let config: PluginConfig = if user_config.is_null() {
                Default::default()
            } else {
                serde_json::from_value(user_config.clone())?
            };

            #[cfg(feature = "sqlite")]
            create_dir_all(app_path(app)).expect("problems creating App directory!");

            let instances = DbInstances::default();
            let mut lock = instances.0.lock().await;
            for db in config.preload {
                #[cfg(feature = "sqlite")]
                let fqdb = path_mapper(app_path(app), &db);
                #[cfg(not(feature = "sqlite"))]
                let fqdb = db.clone();

                #[cfg(not(feature = "mssql"))]
                if !Db::database_exists(&fqdb).await.unwrap_or(false) {
                    Db::create_database(&fqdb).await?;
                }
                let pool = Pool::connect(&fqdb).await?;

                // TODO: currently sqlx does not support migrations for mssql
                #[cfg(not(feature = "mssql"))]
                if let Some(migrations) = self.migrations.as_mut().unwrap().remove(&db) {
                    let migrator = Migrator::new(migrations).await?;
                    migrator.run(&pool).await?;
                }

                lock.insert(db, pool);
            }
            drop(lock);
            app.manage(instances);
            #[cfg(not(feature = "mssql"))]
            app.manage(Migrations(Mutex::new(self.migrations.take().unwrap())));

            info!("tauri-sql-plugin is initialized: [config: {}]", user_config);
            Ok(())
        })
    }

    fn extend_api(&mut self, message: Invoke<R>) {
        (self.invoke_handler)(message)
    }

    /// gracefully close all DB pools on application exit
    fn on_event(&mut self, app: &AppHandle<R>, event: &RunEvent) {
        info!("closing all DB pools due to application exit");
        if let RunEvent::Exit = event {
            tauri::async_runtime::block_on(async move {
                let instances = &*app.state::<DbInstances>();
                let instances = instances.0.lock().await;
                for value in instances.values() {
                    value.close().await;
                }
            });
        }
    }
}
