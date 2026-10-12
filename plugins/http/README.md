![plugin-http](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/http/banner.png)

Access the HTTP client written in Rust.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | ✓         |
| iOS      | ✓         |

## Install

_This plugin requires a Rust version of at least **1.90**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-http = "2.0.0"
# alternatively with Git:
tauri-plugin-http = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

```sh
pnpm add @tauri-apps/plugin-http
# or
npm add @tauri-apps/plugin-http
# or
yarn add @tauri-apps/plugin-http
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_http::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { fetch } from '@tauri-apps/plugin-http'
const response = await fetch('http://localhost:3003/users/2', {
  method: 'GET',
  connectTimeout: 30
})
```

### Request identity

By default, requests are sent with a `tauri-plugin-http/<version>` User-Agent and with the webview's origin, which depends on the platform (`tauri://localhost` or `http://tauri.localhost`) and on the dev server URL during development. To send a fixed identity instead, configure `userAgent` and `requestHeaders` in `tauri.conf.json`:

```json
{
  "plugins": {
    "http": {
      "userAgent": "my-app/1.0",
      "requestHeaders": [
        {
          "url": "https://api.example.com/*",
          "headers": { "Origin": "https://desktop.example.com" }
        },
        {
          "url": "https://*.thirdparty.example/*",
          "headers": { "Origin": null },
          "mode": "override"
        }
      ]
    }
  }
}
```

Every rule whose `url` pattern matches applies, in order. A string sets the header and `null` removes it. By default a rule only replaces headers the frontend did not set; `"mode": "override"` always applies it. Rules cannot set `Host`, `Content-Length`, `Transfer-Encoding` or `Cookie`.

The patterns only choose which requests get the headers: they do not grant access, which is still controlled by the `http:default` scope. The plugin applies the rules again to every redirect it follows, matched against the redirect target, so headers set for one URL are not sent to another. Since rules can set headers such as `Origin` that the frontend is not allowed to send, review changes to them like a capability change.

The same options are available in Rust, together with a hook for policies the configuration cannot express:

```rust
use tauri_plugin_http::reqwest::header::{HeaderValue, ORIGIN};

tauri::Builder::default()
    .plugin(
        tauri_plugin_http::Builder::new()
            .user_agent("my-app/1.0")
            .on_request(|ctx, headers| {
                if ctx.url().host_str() == Some("api.example.com") && !ctx.frontend_set(&ORIGIN) {
                    headers.insert(ORIGIN, HeaderValue::from_static("https://desktop.example.com"));
                }
            })
            .build(),
    )
```

### Sharing cookies with Rust

The cookie jar used by webview requests can be given to your own `reqwest` clients, so both share one session:

```rust
use tauri_plugin_http::HttpExt;

let jar = app.http().cookie_jar();
let client = tauri_plugin_http::reqwest::Client::builder()
    .cookie_provider(jar.clone())
    .build()?;

jar.clear(); // e.g. on logout
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
