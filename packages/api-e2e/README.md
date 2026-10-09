# Plugins end-to-end tests

WebdriverIO suite that exercises the JavaScript API of the plugins in this
repository against a **real** Tauri app — the [`examples/api`](../../examples/api)
validation app — rather than a mocked backend, on desktop (Linux, macOS, Windows)
and mobile (Android, iOS). Each plugin has its own spec file, shared by every
platform, and adding coverage for a new plugin API is normally just dropping in
one more spec.

It mirrors the [`@tauri-apps/api` e2e suite](https://github.com/tauri-apps/tauri/tree/dev/packages/api-e2e)
in the core repository; the `@tauri-apps/api` modules themselves are covered there.

## How it works

- The example app is built with `withGlobalTauri: true`, so the `@tauri-apps/api` surface
  is reachable on `window.__TAURI__` inside the webview, and every plugin's `api-iife.js`
  registers its API next to it (`window.__TAURI__.fs`, `window.__TAURI__.clipboardManager`,
  …, the package name without the `@tauri-apps/plugin-` prefix, camel-cased).
- On desktop, WebdriverIO drives the app through [`@crabnebula/tauri-driver`](https://www.npmjs.com/package/@crabnebula/tauri-driver),
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
- On mobile, WebdriverIO drives the app through [Appium](https://appium.io) (started by
  `@wdio/appium-service`; the drivers are plain devDependencies of this package, which Appium
  picks up on its own):
  - **Android** — the UiAutomator2 driver. The suite switches to the app's `WEBVIEW_*`
    context, which chromedriver reaches through the WebView's debugging socket. Debug builds
    turn that on (`setWebContentsDebuggingEnabled`), so the suite builds a debug APK. A
    chromedriver matching the device's WebView is downloaded on demand (see `E2E_CHROMEDRIVER`).
  - **iOS** — the XCUITest driver on a simulator, attaching to the WKWebView through the
    WebKit remote inspector. Debug builds mark the webview `isInspectable`, so the suite
    builds an unsigned debug simulator app. The inspector identifies an app by the
    `application-identifier` entitlement that Xcode embeds when it code signs a simulator
    build; an unsigned one has none and is listed as `process-<executable name>` instead of
    its bundle identifier, so the config has the driver match that name too
    (`appium:additionalWebviewBundleIds`). The driver also starts with a script timeout of
    0, which the config raises to the 30s the other drivers default to, or every
    `executeAsync` would time out at once.
- Specs never `eval` in the page. They pass a function to the [`tauri()`](test/helpers/index.ts)
  helper, which serializes it and runs it via the driver's own (CSP-exempt) script injection,
  handing it `window.__TAURI__` as the first argument and returning its JSON result.
- Each spec file gets its own session — a fresh `tauri-driver` (and therefore a fresh app
  instance) on desktop, a fresh Appium session (which relaunches the app) on mobile — so
  each plugin's suite runs in isolation.
- A small [fixture server](test/helpers/server.ts) is started for the whole run. It serves
  the updater manifest the desktop e2e build points the updater at (see
  [`tauri.e2e.conf.json`](tauri.e2e.conf.json)) and the upload/download fixtures. It listens
  on the host's loopback; the iOS simulator shares that network stack, and on Android the
  mobile config runs `adb reverse` so the same `127.0.0.1` URL works on the device. The
  `http` specs use the echo server the example app itself spawns on port 3003, since that
  is the only `http://` origin in the example's http scope. The fixture server also has a
  WebSocket echo endpoint (`ws://127.0.0.1:3004/ws`) for the `websocket` specs.

## What is (not) covered

The example only registers each plugin on the platforms it supports, so the suites
split three ways: desktop-only plugins are skipped on mobile, mobile-only plugins are
skipped on desktop, and the rest run everywhere with the odd test gated.

| Plugin              | Coverage                                                                                                                                                               | On mobile                                                                                |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| `autostart`         | enable/disable/isEnabled, restoring the host's original state.                                                                                                         | Skipped — desktop-only plugin.                                                           |
| `cli`               | `getMatches` shape for an argument-less launch.                                                                                                                        | Skipped — desktop-only plugin.                                                           |
| `clipboard-manager` | text, HTML and image round-trips, `clear`, error paths.                                                                                                                | Text only; HTML and images are unsupported there.                                        |
| `deep-link`         | `getCurrent` on a plain launch, `onOpenUrl` delivery, runtime `register`/`isRegistered`/`unregister` on Linux and Windows.                                             | Only that runtime registration is unsupported.                                           |
| `dialog`            | Only that the API is registered — every dialog blocks on native UI the driver cannot operate.                                                                          | Same.                                                                                    |
| `fs`                | read/write/stat/copy/rename/remove/truncate, `FileHandle`, line iteration, watchers, scope.                                                                            | All but the watchers (`fs:allow-watch` is desktop-only here).                            |
| `global-shortcut`   | register/unregister/isRegistered/unregisterAll and error paths. Shortcuts cannot be triggered.                                                                         | Skipped — desktop-only plugin.                                                           |
| `http`              | `fetch` methods, headers, JSON/bytes/multipart bodies, the cookie jar, abort, scope, failures.                                                                         | Same.                                                                                    |
| `log`               | `attachLogger`/`attachConsole` for webview and Rust records, level filtering.                                                                                          | Same.                                                                                    |
| `notification`      | Permission model and the `window.Notification` override. Sending is fire-and-forget and display is not observable.                                                     | Action types, pending/active lists after cancel/remove, channels, listeners — see below. |
| `opener`            | Scope enforcement only — a successful open launches an external app the suite cannot close.                                                                            | Same.                                                                                    |
| `os`                | Every function but `version` (which has nothing to compare against), checked against what the app was built for and the host.                                          | Same, minus `hostname`.                                                                  |
| `positioner`        | Screen and tray positions (with a tray rect handed in through `handleIconState`), `moveWindowConstrained`, the missing-tray error (WM dependent).                      | Skipped — desktop-only plugin.                                                           |
| `process`           | Only that the API is registered — `exit`/`relaunch` terminate the app under test.                                                                                      | Same.                                                                                    |
| `shell`             | `execute`, `spawn` with stdout/stderr/close events, stdin, `kill`, scope enforcement.                                                                                  | Scope only on iOS, which cannot spawn a process at all.                                  |
| `sql`               | SQLite `load`/`execute`/`select`/`close`, bound values, column types, app-registered migrations, error paths.                                                          | Same.                                                                                    |
| `store`             | CRUD, persistence, auto-save, defaults/reset, reload, `getStore`, `LazyStore`, change events.                                                                          | Same.                                                                                    |
| `updater`           | `check` against the fixture manifest (update / 204 / older release). Installing is never exercised.                                                                    | Skipped — desktop-only plugin.                                                           |
| `upload`            | `download` and `upload` with progress, methods, headers and error paths.                                                                                               | Same (through `adb reverse` on Android).                                                 |
| `websocket`         | Text/binary echo, ping/pong, handshake headers, listener removal, server and client close, connection and argument errors, against the fixture server.                 | Same (through `adb reverse` on Android).                                                 |
| `window-state`      | `filename`, save/restore, and that a re-created window gets its saved size (WM dependent).                                                                             | Skipped — desktop-only plugin.                                                           |
| mobile-only plugins | `barcode-scanner`, `biometric`, `geolocation`, `haptics` and `nfc` need hardware or native UI the driver cannot operate, so only their global API surface is asserted. | Only there.                                                                              |

The notification permission specs are desktop-only: a mobile app starts out ungranted and
`requestPermission` puts up a system dialog the session would then block on. The rest of the
notification API (action types, channels, the pending/active lists, listeners) only exists on
mobile, and is covered there without the permission.

The [`plugins.spec.ts`](test/specs/plugins.spec.ts) spec additionally asserts that every
plugin the platform registers injects its global API with its documented members, and that
the _other_ platform's plugins are absent. That is what catches a plugin whose
`global_api_script_path` is missing from its `build.rs`, or a crate that is not target-gated
in the example's `Cargo.toml`.

## Prerequisites

```sh
# from the repo root
pnpm install
pnpm build        # examples/api resolves the plugins' JS from their dist-js output
```

Platform driver dependencies:

| Platform | Requirement                                                                                                                                                                                                                                               |
| -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| macOS    | `CN_API_KEY` env var (CrabNebula Cloud). The automation plugin and test-runner-backend are wired up already.                                                                                                                                              |
| Linux    | `webkit2gtk-driver` package (provides `WebKitWebDriver`).                                                                                                                                                                                                 |
| Windows  | `msedgedriver.exe` matching your Edge version, on `PATH`. Run the suite unelevated.                                                                                                                                                                       |
| Android  | The usual Tauri Android setup (`ANDROID_HOME`, `NDK_HOME`, a JDK), which also provides the `adb` the suite shells out to, plus a running emulator or a connected device with USB debugging. Network access the first time, for the chromedriver download. |
| iOS      | macOS with Xcode and an iOS simulator runtime. The first session compiles WebDriverAgent (a few minutes).                                                                                                                                                 |

## Running

```sh
# desktop, from the repo root
pnpm test:api-e2e

# or from this package
pnpm e2e

# iterate without rebuilding the app every run
E2E_SKIP_BUILD=1 pnpm e2e

# run a single plugin's spec
pnpm exec wdio run ./wdio.conf.ts --spec test/specs/fs.spec.ts

# mobile (from the repo root; or `pnpm e2e:android` / `pnpm e2e:ios` from this package)
pnpm test:api-e2e:android
pnpm test:api-e2e:ios
```

The first desktop run builds the app with [`tauri.e2e.conf.json`](tauri.e2e.conf.json) as a
config override, which enables the example's `automation` feature and points the updater at
the fixture server; afterwards use `E2E_SKIP_BUILD=1` to reuse the existing binary. A binary
supplied through `E2E_SKIP_BUILD` or `E2E_APP_PATH` must have been built with that override:
the `updater` specs rely on its endpoint, and the CrabNebula Webdriver (always on macOS)
relies on the automation feature.

The mobile configs ([`wdio.android.conf.ts`](wdio.android.conf.ts), [`wdio.ios.conf.ts`](wdio.ios.conf.ts),
sharing [`wdio.mobile.ts`](wdio.mobile.ts)) run `tauri android build --debug --apk` /
`tauri ios build --debug --target aarch64-sim --no-sign`, compiling only the Rust target the
device runs (the Android one is read from the connected device through `adb`). The Android
Studio and Xcode projects are committed under `examples/api/src-tauri/gen`, so they are only
initialized if that directory is missing. The app must be a **debug** build — release builds
have webview debugging off, and Appium cannot see the page. `E2E_SKIP_BUILD` and
`E2E_APP_PATH` (an `.apk` / simulator `.app`) work as on desktop. No config override is
passed there: both things it turns on belong to desktop-only plugins.

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

Mobile only:

| Variable             | Purpose                                                                                              |
| -------------------- | ---------------------------------------------------------------------------------------------------- |
| `E2E_ANDROID_TARGET` | Rust target for the APK (`aarch64`, `armv7`, `i686`, `x86_64`); default: the connected device's ABI. |
| `E2E_ANDROID_DEVICE` | `adb` serial of the device/emulator to use (`appium:udid`); default: the first connected one.        |
| `E2E_ANDROID_AVD`    | Name of an AVD for Appium to boot (`appium:avd`) instead of using an already-running emulator.       |
| `E2E_CHROMEDRIVER`   | chromedriver binary matching the device's WebView, instead of letting Appium download one.           |
| `E2E_IOS_TARGET`     | Rust target for the simulator app (`aarch64-sim` or `x86_64`); default: the host architecture.       |
| `E2E_IOS_DEVICE`     | Simulator UDID or name (as in `xcrun simctl list`); default: a booted iPhone, else the newest one.   |
| `E2E_PLATFORM`       | Set by the mobile configs for the spec workers (`android`/`ios`) — see `platform` in the helpers.    |

Appium's own log is written to `logs/wdio-appium.log` in this package.

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
   `devDependencies` and its namespace to the matching interface in
   [`test/helpers/index.ts`](test/helpers/index.ts) — `CommonPluginApi`, `DesktopPluginApi` or
   `MobilePluginApi` — plus the member list in `plugins.spec.ts`.

2. **Grant permissions if needed.** If the API calls a command that the example does not
   allow yet, add the permission to
   [`examples/api/src-tauri/capabilities/base.json`](../../examples/api/src-tauri/capabilities/base.json)
   (or `desktop.json`/`mobile.json` for platform-specific plugins). Each plugin documents its
   permissions under `plugins/<plugin>/permissions/autogenerated/reference.md`.

3. **Need a server?** Add a route to the [fixture server](test/helpers/server.ts) rather than
   hitting the network; its URL is exported as `FIXTURE_SERVER_URL`.

4. **Handle environment-sensitive cases.** Use `itWm` (instead of `it`) for assertions that
   depend on a real window manager, and `eventually()` to poll for state that is applied
   asynchronously. Branch on `platform` from the helpers (never `process.platform`, which is
   the host running the emulator/simulator on mobile) for platform-specific behavior.
   Files go under `scratchDir('<plugin>')`, relative to `BaseDirectory.AppData`, which the
   example's fs scope allows.

5. **Gate what mobile does not have.** The same specs run on Android and iOS. Wrap tests of
   desktop-only behavior — a command the mobile build does not expose, a mobile implementation
   that answers "Unsupported on this platform", or a permission only the desktop capability
   grants — in `itDesktop`, use `itOn('android', …)` / `itOn('ios', …)` for platform-specific
   APIs, and pass `{ desktopOnly: true }` (or `{ mobileOnly: true }`) to `describePlugin` for
   plugins the example does not register on the other side. Skipped tests show up as pending
   rather than silently disappearing.

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
