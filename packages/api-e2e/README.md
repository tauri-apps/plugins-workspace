# Plugins end-to-end tests

WebdriverIO suite that exercises the JavaScript API of every desktop plugin in this
repository against a **real** Tauri app — the [`examples/api`](../../examples/api)
validation app — rather than a mocked backend. Each plugin has its own spec file, and
adding coverage for a new plugin API is normally just dropping in one more spec.

It mirrors the [`@tauri-apps/api` e2e suite](https://github.com/tauri-apps/tauri/tree/dev/packages/api-e2e)
in the core repository; the `@tauri-apps/api` modules themselves are covered there.

## How it works

- The example app is built with `withGlobalTauri: true`, so the `@tauri-apps/api` surface
  is reachable on `window.__TAURI__` inside the webview, and every plugin's `api-iife.js`
  registers its API next to it (`window.__TAURI__.fs`, `window.__TAURI__.clipboardManager`,
  …, the package name without the `@tauri-apps/plugin-` prefix, camel-cased).
- WebdriverIO drives the app through [`@crabnebula/tauri-driver`](https://www.npmjs.com/package/@crabnebula/tauri-driver),
  which bridges the WebDriver protocol to each platform's webview:
  - **macOS** — the CrabNebula Webdriver, which needs [`tauri-plugin-automation`](https://crates.io/crates/tauri-plugin-automation)
    (registered in `examples/api` behind its off-by-default `automation` Cargo feature, which
    the suite's build enables) and a locally-running `@crabnebula/test-runner-backend`,
    authenticated with `CN_API_KEY`.
  - **Linux** — `webkit2gtk-driver` (`WebKitWebDriver` on `PATH`).
  - **Windows** — `msedgedriver.exe` on `PATH`. It hands the app the `--remote-debugging-port`
    it attaches to through `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`, which WebView2 ignores in an
    elevated process ([wry#1782](https://github.com/tauri-apps/wry/issues/1782)), so the suite has
    to run unelevated.
- Specs never `eval` in the page. They pass a function to the [`tauri()`](test/helpers/index.ts)
  helper, which serializes it and runs it via the driver's own (CSP-exempt) script injection,
  handing it `window.__TAURI__` as the first argument and returning its JSON result.
- A fresh `tauri-driver` (and therefore a fresh app instance) is started per spec file, so
  each plugin's suite runs in isolation.
- A small [fixture server](test/helpers/server.ts) is started for the whole run. It serves
  the updater manifest the e2e build points the updater at (see
  [`tauri.e2e.conf.json`](tauri.e2e.conf.json)) and the upload/download fixtures. The
  `http` specs use the echo server the example app itself spawns on port 3003, since that
  is the only `http://` origin in the example's http scope.

## What is (not) covered

| Plugin              | Coverage                                                                                         |
| ------------------- | ------------------------------------------------------------------------------------------------ |
| `cli`               | `getMatches` shape for an argument-less launch.                                                  |
| `clipboard-manager` | text, HTML and image round-trips, `clear`, error paths.                                          |
| `dialog`            | Only that the API is registered — every dialog blocks on native UI the driver cannot operate.    |
| `fs`                | read/write/stat/copy/rename/remove/truncate, `FileHandle`, line iteration, watchers, scope.      |
| `global-shortcut`   | register/unregister/isRegistered/unregisterAll and error paths. Shortcuts cannot be triggered.   |
| `http`              | `fetch` methods, headers, JSON/bytes/multipart bodies, the cookie jar, abort, scope, failures.   |
| `log`               | `attachLogger`/`attachConsole` for webview and Rust records, level filtering.                    |
| `notification`      | Permission model, `window.Notification` override, `sendNotification`. Display is not observable. |
| `opener`            | Scope enforcement only — a successful open launches an external app the suite cannot close.      |
| `os`                | Every function, compared against Node's view of the host.                                        |
| `process`           | Only that the API is registered — `exit`/`relaunch` terminate the app under test.                |
| `shell`             | `execute`, `spawn` with stdout/stderr/close events, stdin, `kill`, scope enforcement.            |
| `store`             | CRUD, persistence, auto-save, defaults/reset, reload, `getStore`, `LazyStore`, change events.    |
| `updater`           | `check` against the fixture manifest (update / 204 / downgrade). Installing is never exercised.  |
| `upload`            | `download` and `upload` with progress, methods, headers and error paths.                         |
| `window-state`      | `filename`, save/restore, and that a re-created window gets its saved size (WM dependent).       |

The [`plugins.spec.ts`](test/specs/plugins.spec.ts) spec additionally asserts that every
plugin's global API is injected with its documented members, which is what catches a plugin
whose `global_api_script_path` is missing from its `build.rs`.

## Prerequisites

```sh
# from the repo root
pnpm install
pnpm build        # examples/api resolves the plugins' JS from their dist-js output
```

Platform driver dependencies:

| Platform | Requirement                                                                                                  |
| -------- | ------------------------------------------------------------------------------------------------------------ |
| macOS    | `CN_API_KEY` env var (CrabNebula Cloud). The automation plugin and test-runner-backend are wired up already. |
| Linux    | `webkit2gtk-driver` package (provides `WebKitWebDriver`).                                                    |
| Windows  | `msedgedriver.exe` matching your Edge version, on `PATH`. Run the suite unelevated.                          |

## Running

```sh
# from the repo root
pnpm test:api-e2e

# or from this package
pnpm e2e

# iterate without rebuilding the app every run
E2E_SKIP_BUILD=1 pnpm e2e

# run a single plugin's spec
pnpm exec wdio run ./wdio.conf.ts --spec test/specs/fs.spec.ts
```

The first run builds the app with [`tauri.e2e.conf.json`](tauri.e2e.conf.json) as a config
override, which enables the example's `automation` feature and points the updater at the
fixture server; afterwards use `E2E_SKIP_BUILD=1` to reuse the existing binary. A binary
supplied through `E2E_SKIP_BUILD` or `E2E_APP_PATH` must have been built with that override:
the `updater` specs rely on its endpoint, and the CrabNebula Webdriver (always on macOS)
relies on the automation feature.

## Environment variables

| Variable            | Purpose                                                                                  |
| ------------------- | ---------------------------------------------------------------------------------------- |
| `CN_API_KEY`        | CrabNebula Cloud key. Required on macOS (and whenever `E2E_CN_WEBDRIVER=1`).             |
| `E2E_SKIP_BUILD`    | Skip the `tauri build` step and reuse the existing binary.                               |
| `E2E_APP_PATH`      | Absolute path to a prebuilt app/binary to test (also implies skip-build).                |
| `E2E_SKIP`          | Comma-separated plugin names to skip, e.g. `E2E_SKIP=clipboard-manager,global-shortcut`. |
| `E2E_SKIP_WM`       | Skip window-manager-dependent tests (the window-state size restore).                     |
| `E2E_SPEC_RETRIES`  | Retry count for flaky spec files (default `0`).                                          |
| `E2E_CN_WEBDRIVER`  | Use the CrabNebula Webdriver on Linux/Windows too (instead of the native driver).        |
| `E2E_NATIVE_DRIVER` | Path passed to `tauri-driver --native-driver` (e.g. a specific chromedriver).            |
| `CARGO_TARGET_DIR`  | Override the target dir the app binary is looked up in.                                  |

## Adding tests for a new plugin API

1. **Add a spec.** Create `test/specs/<plugin>.spec.ts` and use `describePlugin('<plugin>', …)`
   with the `tauri()` helper. It is picked up automatically by the `test/specs/**/*.spec.ts`
   glob. Minimal example:

   ```ts
   import { expect } from '@wdio/globals'
   import { tauri, describePlugin } from '../helpers/index.js'

   describePlugin('os', () => {
     it('reports the platform', async () => {
       expect(await tauri((api) => api.os.platform())).toBe('linux')
     })
   })
   ```

   If the plugin is new to the example, register it in
   [`examples/api/src-tauri/src/lib.rs`](../../examples/api/src-tauri/src/lib.rs), add it to
   the example's `package.json`/`Cargo.toml`, add its `workspace:*` package to this package's
   `devDependencies` and its namespace to the `PluginApi` type in
   [`test/helpers/index.ts`](test/helpers/index.ts) (plus the member list in
   `plugins.spec.ts`).

2. **Grant permissions if needed.** If the API calls a command that the example does not
   allow yet, add the permission to
   [`examples/api/src-tauri/capabilities/base.json`](../../examples/api/src-tauri/capabilities/base.json)
   (or `desktop.json` for desktop-only plugins). Each plugin documents its permissions under
   `plugins/<plugin>/permissions/autogenerated/reference.md`.

3. **Need a server?** Add a route to the [fixture server](test/helpers/server.ts) rather than
   hitting the network; its URL is exported as `FIXTURE_SERVER_URL`.

4. **Handle environment-sensitive cases.** Use `itWm` (instead of `it`) for assertions that
   depend on a real window manager, and `eventually()` to poll for state that is applied
   asynchronously. Branch on `process.platform` (Node side) for platform-specific behavior.
   Files go under `scratchDir('<plugin>')`, relative to `BaseDirectory.AppData`, which the
   example's fs scope allows.

### Rules for `tauri()` page functions

The function you pass to `tauri()` runs **inside the webview**, serialized as a string:

- It **cannot** close over anything from the spec module — pass every value it needs through
  the trailing `tauri(fn, ...args)` arguments.
- It may only reference `api` (the `window.__TAURI__` object), those args, and browser
  globals (`window`, `document`, `setTimeout`, `Promise`, …).
- Its return value must be JSON-serializable — return plain objects/primitives, not class
  instances (call methods and return their results instead), and remember that `undefined`
  values are dropped.
- Restore any app state you mutate (clipboard, registered shortcuts, window size, …); tests
  within a spec file share the same app instance.
- For in-page waiting, wrap logic in a `Promise` with an explicit `setTimeout` rejection so a
  failure surfaces as a message rather than an opaque driver timeout.

Use `tauriError(fn, ...args)` to assert that a call rejects; it returns the rejection message.
