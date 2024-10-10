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

_This plugin requires a Rust version of at least **1.77.2**_

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

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add @tauri-apps/plugin-updater
# or
npm add @tauri-apps/plugin-updater
# or
yarn add @tauri-apps/plugin-updater

# alternatively with Git:
pnpm add https://github.com/tauri-apps/tauri-plugin-updater#v2
# or
npm add https://github.com/tauri-apps/tauri-plugin-updater#v2
# or
yarn add https://github.com/tauri-apps/tauri-plugin-updater#v2
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/main.rs`

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
if (update?.available) {
  await update.downloadAndInstall()
  await relaunch()
}
```

### Config
First, you need to prepare a distribution key pair, you can follow the [V1 guide](https://tauri.app/zh-cn/v1/guides/distribution/updater/) at this time.  
Then, set `TAURI_SIGNING_PRIVATE_KEY` to your private key file , if the private key file have a password,set `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` to your private key file password.(The environment variables are different than v1).  
Next, change your tauri.conf.json file like below. 
```json
{
//"productName": "WindKnight",
"plugins": {
    "updater": {
      "active": true,
      "endpoints": "<your update server or json file url>",
      "dialog": true,
      "pubkey": "<your pub key>"
    }
  },
}
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
