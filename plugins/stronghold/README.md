![plugin-stronghold](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/stronghold/banner.png)

Store secrets and keys using the [IOTA Stronghold](https://github.com/iotaledger/stronghold.rs) secret management engine.

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
tauri-plugin-stronghold = "2.0.0"
# alternatively with Git:
tauri-plugin-stronghold = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: If your JavaScript package manager cannot install packages from git monorepos, you can still use the code by manually copying the [Guest bindings](./guest-js/index.ts) into your source files.

```sh
pnpm add @tauri-apps/plugin-stronghold
# or
npm add @tauri-apps/plugin-stronghold
# or
yarn add @tauri-apps/plugin-stronghold

# alternatively with Git:
pnpm add https://github.com/tauri-apps/tauri-plugin-stronghold#v2
# or
npm add https://github.com/tauri-apps/tauri-plugin-stronghold#v2
# or
yarn add https://github.com/tauri-apps/tauri-plugin-stronghold#v2
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_stronghold::Builder::new(|password| {
            // Hash the password here with e.g. argon2, blake2b or any other secure algorithm
            // Here is an example implementation using the `rust-argon2` crate for hashing the password

            use argon2::{hash_raw, Config, Variant, Version};

            let config = Config {
                lanes: 4,
                mem_cost: 10_000,
                time_cost: 10,
                variant: Variant::Argon2id,
                version: Version::Version13,
                ..Default::default()
            };

            let salt = "your-salt".as_bytes();

            let key = hash_raw(password.as_ref(), salt, &config).expect("failed to hash password");

            key.to_vec()
        })
        .build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { Stronghold, Location, Client } from "tauri-plugin-stronghold-api";
import { appDataDir } from "@tauri-apps/api/path";

const initStronghold = async () => {
  const vaultPath = `${await appDataDir()}/vault.hold`;

  const vaultKey = "The key to the vault";

  const stronghold = await Stronghold.load(vaultPath, vaultKey);

  let client: Client;

  const clientName = "name your client";

  try {
    client = await hold.loadClient(clientName);
  } catch {
    client = await hold.createClient(clientName);
  }

  return {
    stronghold,
    client,
  };
};

const { stronghold, client } = await initStronghold();

const store = client.getStore();

const key = "my_key";

// Insert a record to the store

const data = Array.from(new TextEncoder().encode("Hello, World!"));

await store.insert(key, data);

// Read a record from store

const data = await store.get(key);

const value = new TextDecoder().decode(new Uint8Array(data));

// Save your updates

await stronghold.save();

// Remove a record from store

await store.remove(key);
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
