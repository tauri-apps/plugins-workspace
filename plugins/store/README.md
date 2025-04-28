![plugin-store](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/store/banner.png)

Simple, persistent key-value store.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | ✓         |
| iOS      | ✓         |

## Install

_This plugin requires a Rust version of at least **1.77.2**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-store = "2.0.0"
# alternatively with Git:
tauri-plugin-store = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add @tauri-apps/plugin-store
# or
npm add @tauri-apps/plugin-store
# or
yarn add @tauri-apps/plugin-store

# alternatively with Git:
pnpm add https://github.com/tauri-apps/tauri-plugin-store#v2
# or
npm add https://github.com/tauri-apps/tauri-plugin-store#v2
# or
yarn add https://github.com/tauri-apps/tauri-plugin-store#v2
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```typescript
import { Store } from '@tauri-apps/plugin-store'

const store = await Store.load('settings.json')

await store.set('some-key', { value: 5 })

const val = await store.get<{ value: number }>('some-key')

if (val) {
  console.log(val)
} else {
  console.log('val is null')
}
```

### Persisting Values

Modifications made to the store are automatically saved by default

You can manually save a store with:

```javascript
await store.save()
```

Stores are loaded automatically when used from the JavaScript bindings.  
However, you can also load them manually later like so:

```javascript
await store.load()
```

### LazyStore

There's also a high level API `LazyStore` which only loads the store on first access, note that the options will be ignored if a `Store` with that path has already been created

```typescript
import { LazyStore } from '@tauri-apps/plugin-store'

const store = new LazyStore('settings.json')
```

## Usage from Rust

You can also create `Store` instances directly in Rust:

```rust
use tauri_plugin_store::StoreExt;
use serde_json::json;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            // This loads the store from disk
            let store = app.store("app_data.json")?;

            // Note that values must be serde_json::Value instances,
            // otherwise, they will not be compatible with the JavaScript bindings.
            store.set("a".to_string(), json!("b"));
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Frontend Interoperability

The store created from both Rust side and JavaScript side are stored in the app's resource table and can be accessed by both sides, you can access it by using the same path, with `getStore` and `LazyStore` in the JavaScript side and `get_store` and `store` in the Rust side

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
