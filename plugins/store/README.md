![plugin-store](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/store/banner.png)

Simple, persistent key-value store.

## Install

_This plugin requires a Rust version of at least **1.75**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-store = "2.0.0-rc"
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

`src-tauri/src/main.rs`

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
import { Store } from "tauri-plugin-store-api";

const store = new Store(".settings.dat");

await store.set("some-key", { value: 5 });

const val = await store.get<{ value: number }>("some-key");

if (val) {
  console.log(val);
} else {
  console.log("val is null");
}

// This manually saves the store.
await store.save();
```

### Persisting Values

As seen above, values added to the store are not persisted between application loads unless the application is closed gracefully.

You can manually save a store with:

```javascript
await store.save();
```

Stores are loaded automatically when used from the JavaScript bindings.  
However, you can also load them manually later like so:

```javascript
await store.load();
```

## Usage from Rust

You can also create `Store` instances directly in Rust:

```rust
use tauri_plugin_store::StoreBuilder;
use serde_json::json;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let mut store = StoreBuilder::new("app_data.bin").build(app.handle().clone());

            // Attempt to load the store, if it's saved already.
            store.load().expect("Failed to load store from disk");

            // Note that values must be serde_json::Value instances,
            // otherwise, they will not be compatible with the JavaScript bindings.
            store.insert("a".to_string(), json!("b"));

            // You can manually save the store after making changes.
            // Otherwise, it will save upon graceful exit as described above.
            store.save()
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Loading Gracefully

If you call `load` on a `Store` that hasn't yet been written to the disk, it will return an error. You must handle this error if you want to gracefully continue and use the default store until you save it to the disk. The example above shows how to do this.

For example, this would cause a panic if the store has not yet been created:

```rust
store.load().unwrap();
```

Rather than silently continuing like you may expect.

You should always handle the error appropriately rather than unwrapping, or you may experience unexpected app crashes:

```rust
store.load().expect("Failed to load store from disk");
```

### Frontend Interoperability

As you may have noticed, the `Store` crated above isn't accessible to the frontend. To interoperate with stores created by JavaScript use the exported `with_store` method:

```rust
use tauri::Wry;
use tauri_plugin_store::with_store;

let stores = app.state::<StoreCollection<Wry>>();
let path = PathBuf::from("app_data.bin");

with_store(app_handle, stores, path, |store| store.insert("a".to_string(), json!("b")))
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
