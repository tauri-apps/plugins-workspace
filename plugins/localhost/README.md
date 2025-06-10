![plugin-localhost](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/localhost/banner.png)

Expose your apps assets through a localhost server instead of the default custom protocol.

| Platform | Supported |
| -------- | --------- |
| Linux    | ✓         |
| Windows  | ✓         |
| macOS    | ✓         |
| Android  | ✓         |
| iOS      | ✓         |

> Note: This plugins brings considerable security risks and you should only use it if you know what your are doing. If in doubt, use the default custom protocol implementation.

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
portpicker = "0.1" # used in the example to pick a random free port
tauri-plugin-localhost = "2.0.0"
# alternatively with Git:
tauri-plugin-localhost = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v2" }
```

## Usage

First you need to register the core plugin with Tauri:

`src-tauri/src/lib.rs`

```rust
use tauri::{Manager, window::WindowBuilder, WindowUrl};

fn main() {
  let port = portpicker::pick_unused_port().expect("failed to find unused port");

  tauri::Builder::default()
    .plugin(tauri_plugin_localhost::Builder::new(port).build())
    .setup(move |app| {
      app.ipc_scope().configure_remote_access(
        RemoteDomainAccessScope::new("localhost")
          .add_window("main")
      );

      let url = format!("http://localhost:{}", port).parse().unwrap();
      WindowBuilder::new(app, "main".to_string(), WindowUrl::External(url))
        .title("Localhost Example")
        .build()?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
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
