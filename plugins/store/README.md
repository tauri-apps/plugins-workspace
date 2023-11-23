![plugin-store](https://github.com/tauri-apps/plugins-workspace/raw/v1/plugins/store/banner.png)

Simple, persistent key-value store.

## Install

_This plugin requires a Rust version of at least **1.64**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-store = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v1" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add https://github.com/tauri-apps/tauri-plugin-store#v1
# or
npm add https://github.com/tauri-apps/tauri-plugin-store#v1
# or
yarn add https://github.com/tauri-apps/tauri-plugin-store#v1
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

```javascript
import { Store } from "tauri-plugin-store-api";

const store = new Store(".settings.dat");

await store.set("some-key", { value: 5 });

const val = await store.get("some-key");
assert(val, { value: 5 });

await store.save(); // this manually saves the store, otherwise the store is only saved when your app is closed
```

### Persisting values

Values added to the store are not persisted between application loads unless:

1. The application is closed gracefully (plugin automatically saves)
2. The store is manually saved (using `store.save()`)

## Usage from Rust

You can also access Stores from Rust, you can create new stores:

```rust
use tauri_plugin_store::StoreBuilder;
use serde_json::json;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            let mut store = StoreBuilder::new(app.handle(), "path/to/store.bin".parse()?).build();

            store.insert("a".to_string(), json!("b")) // note that values must be serd_json::Value to be compatible with JS
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

As you may have noticed, the Store crated above isn't accessible to the frontend. To interoperate with stores created by JS use the exported `with_store` method:

```rust
use tauri::Wry;
use tauri_plugin_store::with_store;

let stores = app.state::<StoreCollection<Wry>>();
let path = PathBuf::from("path/to/the/storefile");

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
          <img src="https://github.com/tauri-apps/plugins-workspace/raw/v1/.github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).

## License

Code: (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
