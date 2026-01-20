// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Serialize;
use tauri::{command, AppHandle, Emitter, Runtime, State};

use crate::{DbInstances, DbPool, Error};

#[derive(Clone, Serialize)]
pub struct UpdateHookEvent {
    pub operation: String,
    pub database: String,
    pub table: String,
    pub rowid: i64,
}

#[command]
pub(crate) async fn setup_update_hook<R: Runtime>(
    app: AppHandle<R>,
    db_instances: State<'_, DbInstances>,
    db: String,
) -> Result<(), Error> {
    #[cfg(feature = "sqlite")]
    {
        let instances = db_instances.0.read().await;
        let db_pool = instances
            .get(&db)
            .ok_or_else(|| Error::DatabaseNotLoaded(db.clone()))?;

        let sqlite_pool = match db_pool {
            DbPool::Sqlite(pool) => pool,
            _ => return Err(Error::InvalidDbUrl(
                format!("Cannot setup update hook for {}: update hooks are only supported for SQLite databases", db)
            )),
        };

        sqlite_pool
            .rebuild_pool(Some(move |result: sqlx::sqlite::UpdateHookResult| {
                let operation = match result.operation {
                    sqlx::sqlite::SqliteOperation::Insert => "INSERT",
                    sqlx::sqlite::SqliteOperation::Update => "UPDATE",
                    sqlx::sqlite::SqliteOperation::Delete => "DELETE",
                    sqlx::sqlite::SqliteOperation::Unknown(_) => "UNKNOWN",
                };

                let event = UpdateHookEvent {
                    operation: operation.to_string(),
                    database: result.database.to_string(),
                    table: result.table.to_string(),
                    rowid: result.rowid,
                };

                if let Err(e) = app.emit("sqlite-update-hook", &event) {
                    log::error!("[tauri-plugin-sql] Failed to emit update hook event: {}", e);
                }
            }))
            .await
            .map_err(Error::Sql)?;

        Ok(())
    }
}

#[command]
pub(crate) async fn remove_update_hook(
    db_instances: State<'_, DbInstances>,
    db: String,
) -> Result<(), Error> {
    #[cfg(feature = "sqlite")]
    {
        let instances = db_instances.0.read().await;
        let db_pool = instances
            .get(&db)
            .ok_or_else(|| Error::DatabaseNotLoaded(db.clone()))?;

        let sqlite_pool = match db_pool {
            DbPool::Sqlite(pool) => pool,
            _ => return Err(Error::InvalidDbUrl(
                format!("Cannot remove update hook for {}: update hooks are only supported for SQLite databases", db)
            )),
        };

        sqlite_pool
            .rebuild_pool(None::<fn(sqlx::sqlite::UpdateHookResult)>)
            .await
            .map_err(Error::Sql)?;

        Ok(())
    }

    #[cfg(not(feature = "sqlite"))]
    {
        Err(Error::InvalidDbUrl(
            "Update hooks are only supported for SQLite".to_string(),
        ))
    }
}
