![plugin-log](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/log/banner.png)

Configurable logging for your Tauri app.

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
tauri-plugin-log = "2.0.0"
# alternatively with Git:
tauri-plugin-log = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

If you want the single instance mechanism to only trigger for semver compatible instances of your apps, for example if you expect users to have multiple installations of your app installed, you can add `features = ["semver"]` to the dependency declaration in `Cargo.toml`.

Then you can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add @tauri-apps/plugin-log
# or
npm add @tauri-apps/plugin-log
# or
yarn add @tauri-apps/plugin-log

# alternatively with Git:
pnpm add https://github.com/tauri-apps/tauri-plugin-log#v2
# or
npm add https://github.com/tauri-apps/tauri-plugin-log#v2
# or
yarn add https://github.com/tauri-apps/tauri-plugin-log#v2
```

## Usage

First, you should enable the `log:default` capability:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Capability for the main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "opener:default",
    "log:default" # add this!
  ]
}
```

Then, you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
use tauri_plugin_log::{Target, TargetKind};

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new().targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir { file_name: None }),
            Target::new(TargetKind::Webview),
        ]).build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { trace, info, error, attachConsole } from '@tauri-apps/plugin-log'

// with TargetKind::Webview enabled this function will print logs to the browser console
const detach = await attachConsole()

trace('Trace')
info('Info')
error('Error')

// detach the browser console from the log stream
detach()
```

To log from rust code, add the log crate to your `Cargo.toml`:

```toml
[dependencies]
log = "^0.4"
```

Now, you can use the macros provided by the log crate to log messages from your backend. See the [docs](https://docs.rs/log/latest) for more details.

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
