// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, tauriError, describePlugin } from '../helpers/index.js'

// SQLite databases live in the app config directory. The spec works on its
// own database, recreating its table before each test, and only reads the
// example's `sqlite:api.db` to check the migration the app registers for it.

const db = 'sqlite:e2e-sql.db'

interface Row {
  id: number
  name: string
  score: number | null
  ratio: number | null
}

describePlugin('sql', () => {
  beforeEach(async () => {
    await tauri(async (api, db) => {
      const database = await api.sql.load(db)
      await database.execute('DROP TABLE IF EXISTS e2e')
      await database.execute(
        'CREATE TABLE e2e (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, score INTEGER, ratio REAL)'
      )
    }, db)
  })

  it('load resolves with a database for the given path', async () => {
    const path = await tauri(
      async (api, db) => (await api.sql.load(db)).path,
      db
    )
    expect(path).toBe(db)
  })

  it('migrations registered by the app run when their database is loaded', async () => {
    const result = await tauri(async (api) => {
      const database = await api.sql.load('sqlite:api.db')
      return {
        tables: await database.select<{ name: string }[]>(
          "SELECT name FROM sqlite_master WHERE type = 'table' AND name = 'todos'"
        ),
        migrations: await database.select<
          { version: number; description: string; success: number }[]
        >('SELECT version, description, success FROM _sqlx_migrations')
      }
    })
    expect(result.tables).toEqual([{ name: 'todos' }])
    // `success` is declared BOOLEAN but stored, and read back, as an integer
    expect(result.migrations).toEqual([
      { version: 1, description: 'create_todos_table', success: 1 }
    ])
  })

  it('execute reports affected rows and the last insert id', async () => {
    const results = await tauri(async (api, db) => {
      const database = api.sql.get(db)
      const first = await database.execute(
        'INSERT INTO e2e (name, score) VALUES ($1, $2)',
        ['first', 1]
      )
      const second = await database.execute(
        'INSERT INTO e2e (name, score) VALUES ($1, $2)',
        ['second', 2]
      )
      const update = await database.execute('UPDATE e2e SET score = score + 10')
      return { first, second, update }
    }, db)
    expect(results.first).toEqual({ rowsAffected: 1, lastInsertId: 1 })
    expect(results.second).toEqual({ rowsAffected: 1, lastInsertId: 2 })
    expect(results.update.rowsAffected).toBe(2)
  })

  it('select returns rows as objects, with bound values and column types', async () => {
    const rows = await tauri(async (api, db) => {
      const database = api.sql.get(db)
      await database.execute(
        'INSERT INTO e2e (name, score, ratio) VALUES ($1, $2, $3), ($4, $5, $6)',
        ['alpha', 42, 0.5, 'beta', null, null]
      )
      return {
        all: await database.select<Row[]>('SELECT * FROM e2e ORDER BY id'),
        filtered: await database.select<Row[]>(
          'SELECT name FROM e2e WHERE score = $1',
          [42]
        ),
        none: await database.select<Row[]>(
          'SELECT * FROM e2e WHERE name = $1',
          ['missing']
        )
      }
    }, db)
    expect(rows.all).toEqual([
      { id: 1, name: 'alpha', score: 42, ratio: 0.5 },
      { id: 2, name: 'beta', score: null, ratio: null }
    ])
    expect(rows.filtered).toEqual([{ name: 'alpha' }])
    expect(rows.none).toEqual([])
  })

  it('column order follows the query', async () => {
    const columns = await tauri(async (api, db) => {
      const database = api.sql.get(db)
      await database.execute("INSERT INTO e2e (name, score) VALUES ('x', 1)")
      const [row] = await database.select<Record<string, unknown>[]>(
        'SELECT score, name, id FROM e2e'
      )
      return Object.keys(row)
    }, db)
    expect(columns).toEqual(['score', 'name', 'id'])
  })

  it('invalid statements reject', async () => {
    const error = await tauriError(
      (api, db) => api.sql.get(db).execute('INSERT INTO missing VALUES (1)'),
      db
    )
    expect(error).toMatch(/no such table: missing/)
    const constraint = await tauriError(
      (api, db) =>
        api.sql.get(db).execute('INSERT INTO e2e (name) VALUES ($1)', [null]),
      db
    )
    expect(constraint).toMatch(/NOT NULL constraint failed/)
  })

  it('a database that was never loaded is rejected', async () => {
    const error = await tauriError((api) =>
      api.sql.get('sqlite:never-loaded.db').select('SELECT 1')
    )
    expect(error).toMatch(/database sqlite:never-loaded\.db not loaded/)
  })

  it('close shuts the pool down until the database is loaded again', async () => {
    const result = await tauri(async (api, db) => {
      const database = api.sql.get(db)
      await database.execute("INSERT INTO e2e (name) VALUES ('persisted')")
      const closed = await database.close(db)
      let error = ''
      try {
        await database.select('SELECT * FROM e2e')
      } catch (e) {
        error = String(e)
      }
      const reloaded = await api.sql.load(db)
      return {
        closed,
        error,
        rows: await reloaded.select<{ name: string }[]>('SELECT name FROM e2e')
      }
    }, db)
    expect(result.closed).toBe(true)
    expect(result.error).toMatch(/closed/i)
    // the data was written to disk, not just to the closed pool
    expect(result.rows).toEqual([{ name: 'persisted' }])
  })

  it('closing a database that was never loaded is rejected', async () => {
    const error = await tauriError((api) =>
      api.sql.get('sqlite:never-loaded.db').close('sqlite:never-loaded.db')
    )
    expect(error).toMatch(/not loaded/)
  })
})
