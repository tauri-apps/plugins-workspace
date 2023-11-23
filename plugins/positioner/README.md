![plugin-positioner](https://github.com/tauri-apps/plugins-workspace/raw/v1/plugins/positioner/banner.png)

Position your windows at well-known locations.

This plugin is a port of [electron-positioner](https://github.com/jenslind/electron-positioner) for Tauri.

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
tauri-plugin-positioner = "1.0"
# or through git
tauri-plugin-positioner = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v1" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add tauri-plugin-positioner-api
# or
npm add tauri-plugin-positioner-api
# or
yarn add tauri-plugin-positioner-api
```

Or through git:

```sh
pnpm add https://github.com/tauri-apps/tauri-plugin-positioner#v1
# or
npm add https://github.com/tauri-apps/tauri-plugin-positioner#v1
# or
yarn add https://github.com/tauri-apps/tauri-plugin-positioner#v1
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        // This is required to get tray-relative positions to work
        .on_system_tray_event(|app, event| {
           tauri_plugin_positioner::on_tray_event(app, &event);
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { move_window, Position } from "tauri-plugin-positioner-api";

move_window(Position.TopRight);
```

If you only intend on moving the window from rust code, you can import the Window trait extension instead of registering the plugin:

```rust
use tauri_plugin_positioner::{WindowExt, Position};

let mut win = app.get_window("main").unwrap();
let _ = win.move_window(Position::TopRight);
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

Code: (c) 2021 - Jonas Kruckenberg. 2021 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
