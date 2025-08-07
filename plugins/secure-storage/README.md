![secure-storage](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/secure-storage/banner.png)

Store data in the platforms' keychains.

<!-- TODO: List the keychains we use -->

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | ✓         |
| iOS      | ✓         |

## Install

<!-- TODO: This will change with keyring v4 -->

_This plugin requires a Rust version of at least **1.77.2**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-secure-storage = "2.0.0"
# alternatively with Git:
tauri-plugin-secure-storage = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-secure-storage
# or
npm add @tauri-apps/plugin-secure-storage
# or
yarn add @tauri-apps/plugin-secure-storage
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_secure_storage::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```typescript
import { setString, getString } from '@tauri-apps/plugin-secure-storage'

await setString('some-key', 'some-secret-value')

const storedData = getString('some-key')

console.log(storedData) // Should return `some-secret-value`
```

Similarily, the plugin also has Rust APIs:

```rs
use tauri_plugin_secure_storage::SecureStorageExt;

app.secure_storage().set_string("some-key", "some-secret-value");

let stored_data = app.secure_storage().get_string("some-key");
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
