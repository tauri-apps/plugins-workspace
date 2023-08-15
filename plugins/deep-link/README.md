![plugin-deep-link](banner.png)

Set your Tauri application as the default handler for an URL.

> Note: This plugins brings considerable security risks and you should only use it if you know what your are doing. If in doubt, use the default custom protocol implementation.

## Install

_This plugin requires a Rust version of at least **1.64**_

There are three general methods of installation that we can recommend.

1. Use crates.io and npm (easiest, and requires you to trust that our publishing pipeline worked)
2. Pull sources directly from Github using git tags / revision hashes (most secure)
3. Git submodule install this repo in your tauri project and then use file protocol to ingest the source (most secure, but inconvenient to use)

Install the Core plugin by adding the following to your `Cargo.toml` file:

`src-tauri/Cargo.toml`

```toml
[dependencies]
tauri-plugin-deep-link = "0.1.2"
# alternatively with Git:
tauri-plugin-deep-link = { git = "https://github.com/tauri-apps/plugins-workspace", branch = "v1" }
```

## Usage

Depending on your use case, for example a `Login with Google` button, you may want to take a look at https://github.com/FabianLars/tauri-plugin-oauth instead. It uses a minimalistic localhost server for the OAuth process instead of custom uri schemes because some oauth providers, like the aforementioned Google, require this setup. Personally, I think it's easier to use too.

Check out the [`example/`](https://github.com/tauri-apps/plugins-workspace/tree/v1/plugins/deep-link/example) directory for a minimal example. You must copy it into an actual tauri app first!

## macOS

In case you're one of the very few people that didn't know this already: macOS hates developers! Not only is that why the macOS implementation took me so long, it also means _you_ have to be a bit more careful if your app targets macOS:

- Read through the methods' platform-specific notes.
- On macOS you need to register the schemes in a `Info.plist` file at build time, the plugin can't change the schemes at runtime.
- macOS apps are in single-instance by default so this plugin will not manually shut down secondary instances in release mode.
  - To make development via `tauri dev` a little bit more pleasant, the plugin will work similar-ish to Linux and Windows _in debug mode_ but you will see secondary instances show on the screen for a split second and the event will trigger twice in the primary instance (one of these events will be an empty string). You still have to install a `.app` bundle you got from `tauri build --debug` for this to work!

## Contributing

PRs accepted. Please make sure to read the Contributing Guide before making a pull request.

## License

Code: (c) 2015 - Present - The Tauri Programme within The Commons Conservancy.

MIT or MIT/Apache 2.0 where applicable.
