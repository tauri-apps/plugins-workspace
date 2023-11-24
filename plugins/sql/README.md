![plugin-sql](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/sql/banner.png)

Interface with SQL databases through [sqlx](https://github.com/launchbadge/sqlx). It supports the `sqlite`, `mysql` and `postgres` drivers, enabled by a Cargo feature.

## Install

_This plugin requires a Rust version of at least **1.70**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies.tauri-plugin-sql]
features = ["sqlite"] # or "postgres", or "mysql"
version = "2.0.0-alpha"
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

`src-tauri/src/main.rs`

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
import Database from "@tauri-apps/plugin-sql";

// sqlite. The path is relative to `tauri::api::path::BaseDirectory::App`.
const db = await Database.load("sqlite:test.db");
// mysql
const db = await Database.load("mysql://user:pass@host/database");
// postgres
const db = await Database.load("postgres://postgres:password@localhost/test");

await db.execute("INSERT INTO ...");
```

## Syntax

We use sqlx as our underlying library, adopting their query syntax:

- sqlite and postgres use the "$#" syntax when substituting query data
- mysql uses "?" when substituting query data

```javascript
// INSERT and UPDATE examples for sqlite and postgres
const result = await db.execute(
  "INSERT into todos (id, title, status) VALUES ($1, $2, $3)",
  [todos.id, todos.title, todos.status],
);

const result = await db.execute(
  "UPDATE todos SET title = $1, completed = $2 WHERE id = $3",
  [todos.title, todos.status, todos.id],
);

// INSERT and UPDATE examples for mysql
const result = await db.execute(
  "INSERT into todos (id, title, status) VALUES (?, ?, ?)",
  [todos.id, todos.title, todos.status],
);

const result = await db.execute(
  "UPDATE todos SET title = ?, completed = ? WHERE id = ?",
  [todos.title, todos.status, todos.id],
);
```

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
