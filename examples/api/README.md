# API example

This example demonstrates Tauri's API capabilities using the plugins from this repository. It's used as the main validation app, serving as the testbed of our development process.
In the future, this app will be used on Tauri's integration tests.

![App screenshot](./screenshot.png?raw=true)

## Running the example

- Install dependencies (Run inside of this folder `examples/api/`)
```bash
# with yarn
$ yarn
# with npm
$ npm install
```

- Run the app in development mode (Run inside of this folder `examples/api/`)
```bash
# with yarn
$ yarn tauri dev
# with npm
$ npm run tauri dev
```

- Build an run the release app (Run inside of this folder `examples/api/`)
```bash
$ yarn tauri build
$ ./src-tauri/target/release/app
```
