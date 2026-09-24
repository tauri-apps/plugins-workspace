![plugin-updater](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/updater/banner.png)

In-app updates for Tauri applications.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | x         |
| iOS      | x         |

## Install

_This plugin requires a Rust version of at least **1.89.0**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
# you can add the dependencies on the `[dependencies]` section if you do not target mobile
[target."cfg(not(any(target_os = \"android\", target_os = \"ios\")))".dependencies]
tauri-plugin-updater = "2.0.0"
# alternatively with Git:
tauri-plugin-updater = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-updater
# or
npm add @tauri-apps/plugin-updater
# or
yarn add @tauri-apps/plugin-updater
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_updater::Builder::new().build())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
const update = await check()
if (update) {
  await update.downloadAndInstall()
  // Relaunch the app on macOS and Linux to run the newly install version
  await relaunch()
}
```

Note that for these APIs to work you have to properly configure the updater first and generate updater artifacts. Please refer to the [guide on our website](https://v2.tauri.app/plugin/updater/) for this.

### Cache behavior

This plugin implements standard HTTP caching when checking for updates.
The cache is kept in-memory, so restarting the application also clears the cache.

By default the plugin strikes a good balance between efficiency and update latency.
But a few key configurations you should consider.

In `tauri.conf.json` you can set global cache preferences:

```jsonc
{
  "plugins": {
    "updater": {
      "cache": {
        // A maximum cache TTL can be set. Helpful if you don't control
        // the server response headers and you need to reduce the cache duration.
        "maxTtlMins": 180
      }
    }
  }
}
```

For each `check()` call you can provide a `cacheMode` option.
This is especially useful if you're creating a "Check for updates now" button for your users.

```javascript
import { check } from '@tauri-apps/plugin-updater'

// Uses the `checkNow` mode to revalidate the update status when the user is asking for a refresh.
const update = await check({ cacheMode: 'checkNow' })
```

Refer to the documentation for all available modes and configuration.

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
