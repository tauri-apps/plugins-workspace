# API example

This example demonstrates Tauri's API capabilities using the plugins from this repository. It's used as the main validation app, serving as the testbed of our development process.
In the future, this app will be used on Tauri's integration tests.

![App screenshot](./screenshot.png?raw=true)

## Running the example

- Install dependencies and build packages (Run inside of the repository root)

```bash
$ pnpm install
$ pnpm build
```

- Run the app in development mode (Run inside of this folder `examples/api/`)

```bash
$ pnpm tauri dev
```

- Build an run the release app (Run inside of this folder `examples/api/`)

```bash
$ pnpm tauri build
$ ./src-tauri/target/release/app
```

## Running on the CEF runtime

The webview runtime is selected at build time by the `wry` (default) and `cef`
features of the example, which pick the `tauri-runtime-wry` or
`tauri-runtime-cef` dependency handed to `tauri::Builder::runtime`. Depending on
`tauri-runtime-cef` is also how the Tauri CLI detects a CEF app, which is what
makes it ship the CEF binary distribution and, on macOS, run the app from inside
an `.app` bundle in `tauri dev`:

```bash
$ pnpm tauri dev --no-default-features --features cef
```

The first build downloads the CEF binary distribution (about 1 GB) into
`{user cache}/tauri-cef`, or into `$CEF_PATH` when that is set. The tray icon is
not set up on CEF, which has no tray integration.
