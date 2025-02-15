![plugin-sql](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/sql/banner.png)

Interface with SQL databases through [sqlx](https://github.com/launchbadge/sqlx). It supports the `sqlite`, `mysql` and `postgres` drivers, enabled by a Cargo feature.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | ✓         |
| iOS      | x         |

## Install

_This plugin requires a Rust version of at least **1.77.2**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies.tauri-plugin-sql]
features = ["sqlite"] # or "postgres", or "mysql"
version = "2.0.0"
# alternatively with Git
git = "https://github.com/tauri-apps/plugins-workspace"
branch = "v2"
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add @tauri-apps/plugin-sql
# or
npm add @tauri-apps/plugin-sql
# or
yarn add @tauri-apps/plugin-sql

# alternatively with Git:
pnpm add https://github.com/tauri-apps/tauri-plugin-sql#v2
# or
npm add https://github.com/tauri-apps/tauri-plugin-sql#v2
# or
yarn add https://github.com/tauri-apps/tauri-plugin-sql#v2
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_sql::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import Database from '@tauri-apps/plugin-sql'

// sqlite. The path is relative to `tauri::api::path::BaseDirectory::AppConfig`.
const db = await Database.load('sqlite:test.db')
// mysql
const db = await Database.load('mysql://user:pass@host/database')
// postgres
const db = await Database.load('postgres://postgres:password@localhost/test')

await db.execute('INSERT INTO ...')
```

## Syntax

We use sqlx as our underlying library, adopting their query syntax:

- sqlite and postgres use the "$#" syntax when substituting query data
- mysql uses "?" when substituting query data

```javascript
// INSERT and UPDATE examples for sqlite and postgres
const result = await db.execute(
  'INSERT into todos (id, title, status) VALUES ($1, $2, $3)',
  [todos.id, todos.title, todos.status]
)

const result = await db.execute(
  'UPDATE todos SET title = $1, status = $2 WHERE id = $3',
  [todos.title, todos.status, todos.id]
)

// INSERT and UPDATE examples for mysql
const result = await db.execute(
  'INSERT into todos (id, title, status) VALUES (?, ?, ?)',
  [todos.id, todos.title, todos.status]
)

const result = await db.execute(
  'UPDATE todos SET title = ?, status = ? WHERE id = ?',
  [todos.title, todos.status, todos.id]
)
```

## Migrations

This plugin supports database migrations, allowing you to manage database schema evolution over time.

### Defining Migrations

Migrations are defined in Rust using the `Migration` struct. Each migration should include a unique version number, a description, the SQL to be executed, and the type of migration (Up or Down).

Example of a migration:

```rust
use tauri_plugin_sql::{Migration, MigrationKind};

let migration = Migration {
    version: 1,
    description: "create_initial_tables",
    sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);",
    kind: MigrationKind::Up,
};
```

### Adding Migrations to the Plugin Builder

Migrations are registered with the `Builder` struct provided by the plugin. Use the `add_migrations` method to add your migrations to the plugin for a specific database connection.

Example of adding migrations:

```rust
use tauri_plugin_sql::{Builder, Migration, MigrationKind};

fn main() {
    let migrations = vec![
        // Define your migrations here
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: "CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT);",
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:mydatabase.db", migrations)
                .build(),
        )
        ...
}
```

### Applying Migrations

To apply the migrations when the plugin is initialized, add the connection string to the `tauri.conf.json` file:

```json
{
  "plugins": {
    "sql": {
      "preload": ["sqlite:mydatabase.db"]
    }
  }
}
```

Alternatively, the client side `load()` also runs the migrations for a given connection string:

```ts
import Database from '@tauri-apps/plugin-sql'
const db = await Database.load('sqlite:mydatabase.db')
```

Ensure that the migrations are defined in the correct order and are safe to run multiple times.

### Migration Management

- **Version Control**: Each migration must have a unique version number. This is crucial for ensuring the migrations are applied in the correct order.
- **Idempotency**: Write migrations in a way that they can be safely re-run without causing errors or unintended consequences.
- **Testing**: Thoroughly test migrations to ensure they work as expected and do not compromise the integrity of your database.

## Contributing

PRs accepted. Please make sure to read the Contributing Guide before making a pull request.

## Partners

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank">
          <img src="https://github.com/tauri-apps/plugins-workspace/raw/v2/.github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).

## License

Code: (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
