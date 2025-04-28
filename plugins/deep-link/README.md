![plugin-deep-link](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/deep-link/banner.png)

Set your Tauri application as the default handler for an URL.

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
tauri-plugin-deep-link = "2.0.0"
# alternatively with Git:
tauri-plugin-deep-link = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

You can install the JavaScript Guest bindings using your preferred JavaScript package manager:

> Note: Since most JavaScript package managers are unable to install packages from git monorepos we provide read-only mirrors of each plugin. This makes installation option 2 more ergonomic to use.

```sh
pnpm add @tauri-apps/plugin-deep-link
# or
npm add @tauri-apps/plugin-deep-link
# or
yarn add @tauri-apps/plugin-deep-link

# alternatively with Git:
pnpm add https://github.com/tauri-apps/tauri-plugin-deep-link#v2
# or
npm add https://github.com/tauri-apps/tauri-plugin-deep-link#v2
# or
yarn add https://github.com/tauri-apps/tauri-plugin-deep-link#v2
```

## Setting up

### Android

For [app links](https://developer.android.com/training/app-links#android-app-links), you need a server with a `.well-known/assetlinks.json` endpoint that must return a text response in the given format:

```
[
  {
    "relation": ["delegate_permission/common.handle_all_urls"],
    "target": {
      "namespace": "android_app",
      "package_name": "$APP_BUNDLE_ID",
      "sha256_cert_fingerprints": [
        $CERT_FINGERPRINT
      ]
    }
  }
]
```

Where `$APP_BUNDLE_ID` is the value defined on `tauri.conf.json > identifier` with `-` replaced with `_` and `$CERT_FINGERPRINT` is a list of SHA256 fingerprints of your app's signing certificates, see [verify android applinks](https://developer.android.com/training/app-links/verify-android-applinks#web-assoc) for more information.

### iOS

For [universal links](https://developer.apple.com/documentation/xcode/allowing-apps-and-websites-to-link-to-your-content?language=objc), you need a server with a `.well-known/apple-app-site-association` endpoint that must return a text response in the given format:

```
{
  "applinks": {
    "details": [
      {
        "appIDs": [ "$DEVELOPMENT_TEAM_ID.$APP_BUNDLE_ID" ],
        "components": [
          {
            "/": "/open/*",
            "comment": "Matches any URL whose path starts with /open/"
          }
        ]
      }
    ]
  }
}
```

Where `$DEVELOPMENT_TEAM_ID` is the value defined on `tauri.conf.json > bundle > iOS > developmentTeam` or the `APPLE_DEVELOPMENT_TEAM` environment variable and `$APP_BUNDLE_ID` is the value defined on `tauri.conf.json > identifier`. See [applinks.details](https://developer.apple.com/documentation/bundleresources/applinks/details) for more information.

To verify if your domain has been properly configured to expose the app associations, you can run the following command:

```sh
curl -v https://app-site-association.cdn-apple.com/a/v1/<host>
```

**The apple-app-site-association file must be served over HTTPS and the response must include the `Content-Type: application/json` header.**

To quickly open an app link on the iOS simulator you can execute `xcrun simctl openurl booted <url>`.

See [supporting associated domains](https://developer.apple.com/documentation/xcode/supporting-associated-domains?language=objc) for more information.

## Configuration

Under `tauri.conf.json > plugins > deep-link`, configure the domains (mobile) and schemes (desktop) you want to associate with your application:

```json
{
  "plugins": {
    "deep-link": {
      "mobile": [
        { "host": "your.website.com", "pathPrefix": ["/open"] },
        { "host": "another.site.br" }
      ],
      "desktop": {
        "schemes": ["something", "my-tauri-app"]
      }
    }
  }
}
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_deep_link::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

Afterwards all the plugin's APIs are available through the JavaScript guest bindings:

```javascript
import { onOpenUrl } from '@tauri-apps/plugin-deep-link'
await onOpenUrl((urls) => {
  console.log('deep link:', urls)
})
```

Note that the Plugin will only emit events on macOS, iOS and Android. On Windows and Linux the OS will spawn a new instance of your app with the URL as a CLI argument. If you want your app to behave on Windows & Linux similar to the other platforms you can use the [single-instance](../single-instance/) plugin with the `deep-link` feature enabled.

## Contributing

PRs accepted. Please make sure to read the Contributing Guide before making a pull request.

## Contributed By

<table>
  <tbody>
    <tr>
      <td align="center" valign="middle">
        <a href="https://crabnebula.dev" target="_blank">
          <img src="contributors/crabnebula.svg" alt="CrabNebula" width="283">
        </a>
      </td>
      <td align="center" valign="middle">
        <a href="https://impierce.com" target="_blank">
            <img src="contributors/impierce.svg" alt="Impierce" width="283" height="90">
        </a>
      </td>
    </tr>
  </tbody>
</table>

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
