// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Interface with SQL databases through [sqlx](https://github.com/launchbadge/sqlx).
 * Which database engines can be used depends on the drivers enabled on the Rust
 * side of the plugin: SQLite, MySQL and PostgreSQL.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * The outcome of a statement run through {@link Database.execute}.
 */
export interface QueryResult {
  /** The number of rows affected by the query. */
  rowsAffected: number
  /**
   * The last inserted `id`.
   *
   * This value is not set for Postgres databases. If the
   * last inserted id is required on Postgres, the `select` function
   * must be used, with a `RETURNING` clause
   * (`INSERT INTO todos (title) VALUES ($1) RETURNING id`).
   */
  lastInsertId?: number
}

/**
 * **Database**
 *
 * The `Database` class serves as the primary interface for
 * communicating with the rust side of the sql plugin.
 *
 * @since 2.0.0
 */
export default class Database {
  /**
   * The connection string identifying the database on the Rust side,
   * for instance `sqlite:test.db`, `mysql://user:pass@host/database`
   * or `postgres://user:pass@host/database`.
   */
  path: string

  /**
   * Creates a `Database` instance for the given connection string without
   * opening a connection to it. Use {@link Database.load} to connect to the
   * database, or {@link Database.get} for a database that is already loaded.
   *
   * @param path The database connection string, such as `sqlite:test.db`.
   *
   * @example
   * ```typescript
   * import Database from '@tauri-apps/plugin-sql'
   * const db = new Database('sqlite:test.db')
   * ```
   */
  constructor(path: string) {
    this.path = path
  }

  /**
   * **load**
   *
   * A static initializer which connects to the underlying database and
   * returns a `Database` instance once a connection to the database is established.
   *
   * # Sqlite
   *
   * The path is relative to `tauri::path::BaseDirectory::App` and must start with `sqlite:`.
   *
   * @example
   * ```typescript
   * import Database from '@tauri-apps/plugin-sql'
   * const db = await Database.load('sqlite:test.db')
   * ```
   *
   * @param path The database connection string, such as `sqlite:test.db`. The database is created if it does not exist yet, and any migration registered for it on the Rust side is run.
   * @returns A promise resolving to a `Database` instance connected to the given database.
   */
  static async load(path: string): Promise<Database> {
    const _path = await invoke<string>('plugin:sql|load', {
      db: path
    })

    return new Database(_path)
  }

  /**
   * **get**
   *
   * A static initializer which synchronously returns an instance of
   * the Database class while deferring the actual database connection
   * until the first invocation or selection on the database.
   *
   * # Sqlite
   *
   * The path is relative to `tauri::path::BaseDirectory::App` and must start with `sqlite:`.
   *
   * @example
   * ```typescript
   * import Database from '@tauri-apps/plugin-sql'
   * const db = Database.get('sqlite:test.db')
   * ```
   *
   * @param path The database connection string, such as `sqlite:test.db`.
   * @returns A `Database` instance bound to the given connection string.
   */
  static get(path: string): Database {
    return new Database(path)
  }

  /**
   * **execute**
   *
   * Passes a SQL expression to the database for execution.
   *
   * @example
   * ```typescript
   * import Database from '@tauri-apps/plugin-sql'
   * const db = await Database.load('sqlite:test.db')
   *
   * // for sqlite & postgres
   * // INSERT example
   * const result = await db.execute(
   *    "INSERT into todos (id, title, status) VALUES ($1, $2, $3)",
   *    [ todos.id, todos.title, todos.status ]
   * );
   * // UPDATE example
   * const result = await db.execute(
   *    "UPDATE todos SET title = $1, completed = $2 WHERE id = $3",
   *    [ todos.title, todos.status, todos.id ]
   * );
   *
   * // for mysql
   * // INSERT example
   * const result = await db.execute(
   *    "INSERT into todos (id, title, status) VALUES (?, ?, ?)",
   *    [ todos.id, todos.title, todos.status ]
   * );
   * // UPDATE example
   * const result = await db.execute(
   *    "UPDATE todos SET title = ?, completed = ? WHERE id = ?",
   *    [ todos.title, todos.status, todos.id ]
   * );
   * ```
   *
   * @param query The SQL statement to run, using `$1`, `$2`, ... placeholders on SQLite and PostgreSQL and `?` placeholders on MySQL.
   * @param bindValues The values bound to the query placeholders, in the order they appear in the statement. Defaults to no values.
   * @returns A promise resolving to the number of rows affected by the statement and the last inserted id.
   */
  async execute(query: string, bindValues?: unknown[]): Promise<QueryResult> {
    const [rowsAffected, lastInsertId] = await invoke<[number, number]>(
      'plugin:sql|execute',
      {
        db: this.path,
        query,
        values: bindValues ?? []
      }
    )
    return {
      lastInsertId,
      rowsAffected
    }
  }

  /**
   * **select**
   *
   * Passes in a SELECT query to the database for execution.
   *
   * @example
   * ```typescript
   * import Database from '@tauri-apps/plugin-sql'
   * const db = await Database.load('sqlite:test.db')
   *
   * // for sqlite & postgres
   * const result = await db.select(
   *    "SELECT * from todos WHERE id = $1", [ id ]
   * );
   *
   * // for mysql
   * const result = await db.select(
   *    "SELECT * from todos WHERE id = ?", [ id ]
   * );
   * ```
   *
   * @param query The SQL query to run, using `$1`, `$2`, ... placeholders on SQLite and PostgreSQL and `?` placeholders on MySQL.
   * @param bindValues The values bound to the query placeholders, in the order they appear in the query. Defaults to no values.
   * @returns A promise resolving to the selected rows, each row being an object keyed by column name.
   */
  async select<T>(query: string, bindValues?: unknown[]): Promise<T> {
    const result = await invoke<T>('plugin:sql|select', {
      db: this.path,
      query,
      values: bindValues ?? []
    })

    return result
  }

  /**
   * **close**
   *
   * Closes the database connection pool.
   *
   * @example
   * ```typescript
   * import Database from '@tauri-apps/plugin-sql'
   * const db = await Database.load('sqlite:test.db')
   * const success = await db.close()
   * ```
   *
   * @param db - Optionally state the name of a database if you are managing more than one. Otherwise, all database pools will be in scope.
   * @returns A promise resolving to `true` once the matching connection pools have been closed.
   */
  async close(db?: string): Promise<boolean> {
    const success = await invoke<boolean>('plugin:sql|close', {
      db
    })
    return success
  }
}
