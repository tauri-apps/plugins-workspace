#[cfg(feature = "sqlite")]
use std::{fs::create_dir_all, path::PathBuf};

use indexmap::IndexMap;
use serde_json::Value as JsonValue;
use sqlx::{
    migrate::{MigrateDatabase, Migrator},
    Any, MySql, Pool, Postgres, Sqlite,
};
use tauri::{command, AppHandle, Manager, Runtime, State};

use crate::{
    path_mapper, DbType, Error, Migrations, MySqlInstances, PostgresInstances, Result,
    SqliteInstances,
};

#[command]
pub(crate) async fn load<R: Runtime>(
    app: AppHandle<R>,
    migrations: State<'_, Migrations>,
    db: String,
) -> Result<String> {
    let dbtype = DbType::try_from(&*db)?;
    let fqdb = path_mapper(app.path().app_config_dir()?, &db)?;

    #[cfg(feature = "sqlite")]
    create_dir_all(fqdb).expect("Problem creating App directory!");

    match dbtype {
        DbType::Sqlite => {
            if !Sqlite::database_exists(&fqdb).await.unwrap_or(false) {
                Sqlite::create_database(&fqdb).await?;
            }

            let pool = Pool::connect(&fqdb).await?;

            if let Some(migrations) = migrations.0.lock().await.remove(&db) {
                let migrator = Migrator::new(migrations).await?;
                migrator.run(&pool).await?;
            }

            app.state::<SqliteInstances>()
                .0
                .lock()
                .await
                .insert(db.clone(), pool);
        }
        DbType::Postgres => {
            if !Postgres::database_exists(&fqdb).await.unwrap_or(false) {
                Postgres::create_database(&fqdb).await?;
            }

            let pool = Pool::connect(&fqdb).await?;

            if let Some(migrations) = migrations.0.lock().await.remove(&db) {
                let migrator = Migrator::new(migrations).await?;
                migrator.run(&pool).await?;
            }

            app.state::<PostgresInstances>()
                .0
                .lock()
                .await
                .insert(db.clone(), pool);
        }
        DbType::MySql => {
            if !MySql::database_exists(&fqdb).await.unwrap_or(false) {
                MySql::create_database(&fqdb).await?;
            }

            let pool = Pool::connect(&fqdb).await?;

            if let Some(migrations) = migrations.0.lock().await.remove(&db) {
                let migrator = Migrator::new(migrations).await?;
                migrator.run(&pool).await?;
            }

            app.state::<MySqlInstances>()
                .0
                .lock()
                .await
                .insert(db.clone(), pool);
        }
    }

    Ok(db)
}

/// Allows the database connection(s) to be closed; if no database
/// name is passed in then _all_ database connection pools will be
/// shut down.
#[command]
pub(crate) async fn close<R: Runtime>(app: AppHandle<R>, db: Option<String>) -> Result<bool> {
    let mut instances = db_instances.0.lock().await;

    let pools = if let Some(db) = db {
        vec![db]
    } else {
        instances.keys().cloned().collect()
    };

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
pub(crate) async fn execute<R: Runtime>(
    app: AppHandle<R>,
    db: String,
    query: String,
    values: Vec<JsonValue>,
) -> Result<(u64, i64)> {
    let mut instances = db_instances.0.lock().await;

    let db = instances.get_mut(&db).ok_or(Error::DatabaseNotLoaded(db))?;
    let mut query = sqlx::query(&query);
    for value in values {
        if value.is_null() {
            query = query.bind(None::<JsonValue>);
        } else if value.is_string() {
            query = query.bind(value.as_str().unwrap().to_owned())
        } else if let Some(number) = value.as_number() {
            query = query.bind(number.as_f64().unwrap_or_default())
        } else {
            query = query.bind(value);
        }
    }
    let result = query.execute(&*db).await?;
    #[cfg(feature = "sqlite")]
    let r = Ok((result.rows_affected(), result.last_insert_rowid()));
    #[cfg(feature = "mysql")]
    let r = Ok((result.rows_affected(), result.last_insert_id()));
    #[cfg(feature = "postgres")]
    let r = Ok((result.rows_affected(), 0));
    r
}

#[command]
pub(crate) async fn select<R: Runtime>(
    app: AppHandle<R>,
    db: String,
    query: String,
    values: Vec<JsonValue>,
) -> Result<Vec<IndexMap<String, JsonValue>>> {
    let db = instances.get_mut(&db).ok_or(Error::DatabaseNotLoaded(db))?;
    let mut query = sqlx::query(&query);
    for value in values {
        if value.is_null() {
            query = query.bind(None::<JsonValue>);
        } else if value.is_string() {
            query = query.bind(value.as_str().unwrap().to_owned())
        } else if let Some(number) = value.as_number() {
            query = query.bind(number.as_f64().unwrap_or_default())
        } else {
            query = query.bind(value);
        }
    }
    let rows = query.fetch_all(&*db).await?;
    let mut values = Vec::new();
    for row in rows {
        let mut value = IndexMap::default();
        for (i, column) in row.columns().iter().enumerate() {
            let v = row.try_get_raw(i)?;

            let v = crate::decode::to_json(v)?;

            value.insert(column.name().to_string(), v);
        }

        values.push(value);
    }

    Ok(values)
}
