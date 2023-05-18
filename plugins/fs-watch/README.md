![plugin-fs-watch](banner.png)

Watch files and directories for changes using [notify](https://github.com/notify-rs/notify).

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
tauri-plugin-fs-watch = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v1" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add https://github.com/tauri-apps/tauri-plugin-fs-watch
# or
npm add https://github.com/tauri-apps/tauri-plugin-fs-watch
# or
yarn add https://github.com/tauri-apps/tauri-plugin-fs-watch
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs_watch::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { watch, watchImmediate } from "tauri-plugin-fs-watch-api";

// can also watch an array of paths
const stopWatching = await watch(
  "/path/to/something",
  (event) => {
    const { type, payload } = event;
  },
  { recursive: true }
);

const stopRawWatcher = await watchImmediate(
  ["/path/a", "/path/b"],
  (event) => {
    const { path, operation, cookie } = event;
  },
  {}
);
```

## Contributing

PRs accepted. Please make sure to read the Contributing Guide before making a pull request.

## License

Code: (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
