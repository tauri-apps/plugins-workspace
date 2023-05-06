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
