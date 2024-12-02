![plugin-positioner](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/positioner/banner.png)

Position your windows at well-known locations.

This plugin is a port of [electron-positioner](https://github.com/jenslind/electron-positioner) for Tauri.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | x         |
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
[dependencies]
tauri-plugin-positioner = "2.0.0"
# alternatively with Git:
tauri-plugin-positioner = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add @tauri-apps/plugin-positioner
# or
npm add @tauri-apps/plugin-positioner
# or
yarn add @tauri-apps/plugin-positioner

# alternatively with Git:
pnpm add https://github.com/tauri-apps/tauri-plugin-positioner#v2
# or
npm add https://github.com/tauri-apps/tauri-plugin-positioner#v2
# or
yarn add https://github.com/tauri-apps/tauri-plugin-positioner#v2
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
use tauri::tray::TrayIconBuilder;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        // This is required to get tray-relative positions to work
        .setup(|app| {
            // note that this will create a new TrayIcon
            TrayIconBuilder::new()
                .on_tray_icon_event(|app, event| {
                    tauri_plugin_positioner::on_tray_event(app.app_handle(), &event);
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Alternatively, you may handle the tray events through JavaScript. Register the plugin as previously noted.

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

And in JavaScript, the `action` passed to the TrayIcon should include the handler.

```javascript
import {
  moveWindow,
  Position,
  handleIconState,
} from "@tauri-apps/plugin-positioner";

const action = async (event: TrayIconEvent) => {
  // add the handle in the action to update the state
  await handleIconState(event);

  if (event.type === "Click") {
    // note this option requires enabling the `tray-icon`
    //   feature in the Cargo.toml
    await moveWindow(Position.TrayLeft);
  }
};

const tray = await TrayIcon.new({ id: "main", action });
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { moveWindow, Position } from '@tauri-apps/plugin-positioner'

moveWindow(Position.TopRight)
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
          <img src="https://github.com/tauri-apps/plugins-workspace/raw/v2/.github/sponsors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
    </tr>
  </tbody>
</table>

For the complete list of sponsors please visit our [website](https://tauri.app#sponsors) and [Open Collective](https://opencollective.com/tauri).

## License

Code: (c) 2021 - Jonas Kruckenberg. 2021 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
