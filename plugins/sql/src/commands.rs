// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use sqlx::migrate::Migrator;
use tauri::{command, AppHandle, Runtime, State};

#[cfg(feature = "sqlite")]
use std::collections::HashMap;

use crate::{DbInstances, DbPool, Error, LastInsertId, Migrations};
#[cfg(feature = "sqlite")]
use crate::SqliteOptions;

#[cfg(not(feature = "sqlite"))]
#[command]
pub(crate) async fn load<R: Runtime>(
    app: AppHandle<R>,
    db_instances: State<'_, DbInstances>,
    migrations: State<'_, Migrations>,
    db: String,
) -> Result<String, crate::Error> {
    let pool = DbPool::connect(&db, &app).await?;

    if let Some(migrations) = migrations.0.lock().await.remove(&db) {
        let migrator = Migrator::new(migrations).await?;
        pool.migrate(&migrator).await?;
    }

    db_instances.0.write().await.insert(db.clone(), pool);

    Ok(db)
}

#[cfg(feature = "sqlite")]
#[command]
pub(crate) async fn load<R: Runtime>(
    app: AppHandle<R>,
    db_instances: State<'_, DbInstances>,
    migrations: State<'_, Migrations>,
    db: String,
    pragmas: Option<HashMap<String, String>>,
) -> Result<String, crate::Error> {
    let sqlite_options = if db.starts_with("sqlite:") {
        let mut options = SqliteOptions::default();

        // Apply pragmas if provided
        if let Some(provided_pragmas) = pragmas {
            options.pragmas.extend(provided_pragmas);
        }

        Some(options)
    } else {
        None
    };

    let pool = DbPool::connect(&db, &app, sqlite_options).await?;

    if let Some(migrations) = migrations.0.lock().await.remove(&db) {
        let migrator = Migrator::new(migrations).await?;
        pool.migrate(&migrator).await?;
    }

    db_instances.0.write().await.insert(db.clone(), pool);

    Ok(db)
}

/// Allows the database connection(s) to be closed; if no database
/// name is passed in then _all_ database connection pools will be
/// shut down.
#[command]
pub(crate) async fn close(
    db_instances: State<'_, DbInstances>,
    db: Option<String>,
) -> Result<bool, crate::Error> {
    let instances = db_instances.0.read().await;

    let pools = if let Some(db) = db {
        vec![db]
    } else {
        instances.keys().cloned().collect()
    };

    for pool in pools {
        let db = instances.get(&pool).ok_or(Error::DatabaseNotLoaded(pool))?;
        db.close().await;
    }

    Ok(true)
}

/// Execute a command against the database
#[command]
pub(crate) async fn execute(
    db_instances: State<'_, DbInstances>,
    db: String,
    query: String,
    values: Vec<JsonValue>,
) -> Result<(u64, LastInsertId), crate::Error> {
    let instances = db_instances.0.read().await;

    let db = instances.get(&db).ok_or(Error::DatabaseNotLoaded(db))?;
    db.execute(query, values).await
}

#[command]
pub(crate) async fn select(
    db_instances: State<'_, DbInstances>,
    db: String,
    query: String,
    values: Vec<JsonValue>,
) -> Result<Vec<IndexMap<String, JsonValue>>, crate::Error> {
    let instances = db_instances.0.read().await;

    let db = instances.get(&db).ok_or(Error::DatabaseNotLoaded(db))?;
    db.select(query, values).await
}

// #[command]
// pub(crate) async fn query(
//     db_instances: State<'_, DbInstances>,
//     db: String,
//     query: String,
//     values: Vec<JsonValue>,
// ) -> Result<Vec<IndexMap<String, JsonValue>>, crate::Error> {
//     let instances = db_instances.0.read().await;

//     let db = instances.get(&db).ok_or(Error::DatabaseNotLoaded(db))?;
//     db.
//     // db.select(query, values).await
// }
