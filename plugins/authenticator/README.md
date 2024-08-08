![plugin-authenticator](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/authenticator/banner.png)

Use hardware security-keys in your Tauri App.

- Supported platforms: Windows, Linux, FreeBSD, NetBSD, OpenBSD, and macOS.

## Install

_This plugin requires a Rust version of at least **1.75**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use the file protocol to ingest the source (most secure, but inconvenient to use)

Install the authenticator plugin by adding the following lines to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
# you can add the dependencies on the `[dependencies]` section if you do not target mobile
[target."cfg(not(any(target_os = \"android\", target_os = \"ios\")))".dependencies]
tauri-plugin-authenticator = "2.0.0-rc"
# alternatively with Git:
tauri-plugin-authenticator = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add @tauri-apps/plugin-authenticator
# or
npm add @tauri-apps/plugin-authenticator
# or
yarn add @tauri-apps/plugin-authenticator
```

Alternatively with Git:

```sh
pnpm add https://github.com/tauri-apps/tauri-plugin-authenticator#v2
# or
npm add https://github.com/tauri-apps/tauri-plugin-authenticator#v2
# or
yarn add https://github.com/tauri-apps/tauri-plugin-authenticator#v2
```

## Usage

First, you need to register the authenticator plugin with Tauri:

`src-tauri/src/main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(desktop)]
            app.handle().plugin(tauri_plugin_authenticator::init())?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards, all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { Authenticator } from "@tauri-apps/plugin-authenticator";

const auth = new Authenticator();
auth.init(); // initialize transports

// generate a 32-bytes long random challenge
const arr = new Uint32Array(32);
window.crypto.getRandomValues(arr);
const b64 = btoa(String.fromCharCode.apply(null, arr));
// web-safe base64
const challenge = b64.replace(/\+/g, "-").replace(/\//g, "_");

const domain = "https://tauri.app";

// attempt to register with the security key
const json = await auth.register(challenge, domain);
const registerResult = JSON.parse(json);

// verify the registration was successful
const r2 = await auth.verifyRegistration(
  challenge,
  app,
  registerResult.registerData,
  registerResult.clientData,
);
const j2 = JSON.parse(r2);

// sign some data
const json = await auth.sign(challenge, app, keyHandle);
const signData = JSON.parse(json);

// verify the signature again
const counter = await auth.verifySignature(
  challenge,
  app,
  signData.signData,
  clientData,
  keyHandle,
  pubkey,
);

if (counter && counter > 0) {
  console.log("SUCCESS!");
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
