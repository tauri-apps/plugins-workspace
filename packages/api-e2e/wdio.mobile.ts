// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Shared WebdriverIO configuration for the mobile suites (`wdio.android.conf.ts`
// and `wdio.ios.conf.ts`). Instead of tauri-driver, the example app is driven
// through Appium: the UiAutomator2 driver (Android; chromedriver attaches to the
// WebView) or the XCUITest driver (iOS simulator; WebKit remote inspector).
// Debug builds enable webview debugging on both platforms, which is what makes
// the `WEBVIEW_*` context — and `browser.executeAsync` inside it — available.

import path from 'node:path'
import fs from 'node:fs'
import { spawnSync, type SpawnSyncReturns } from 'node:child_process'
import { fileURLToPath } from 'node:url'
import {
  startFixtureServer,
  FIXTURE_SERVER_PORT,
  type FixtureServer
} from './test/helpers/server.js'

export type MobilePlatform = 'android' | 'ios'

const dirname = path.dirname(fileURLToPath(import.meta.url))
const repoRoot = path.resolve(dirname, '..', '..')
const appDir = path.join(repoRoot, 'examples', 'api')
const tauriDir = path.join(appDir, 'src-tauri')

/** `identifier` in examples/api's tauri.conf.json. */
const appId = 'com.tauri.api'

/** Where Appium looks for drivers; they are devDependencies of this package. */
process.env.APPIUM_HOME ??= dirname

let fixtureServer: FixtureServer | undefined

export function mobileConfig(platform: MobilePlatform): WebdriverIO.Config {
  const ios = platform === 'ios' ? iosTarget() : undefined

  // Path passed to the driver as `appium:app`.
  const application =
    process.env.E2E_APP_PATH
    ?? (ios
      ? path.join(
          tauriDir,
          'gen',
          'apple',
          'build',
          ios.outputArch,
          'Tauri API.app'
        )
      : path.join(
          tauriDir,
          'gen',
          'android',
          'app',
          'build',
          'outputs',
          'apk',
          'universal',
          'debug',
          'app-universal-debug.apk'
        ))

  return {
    specs: ['./test/specs/**/*.spec.ts'],
    // One device/simulator, one app instance: the suite runs serially.
    maxInstances: 1,
    capabilities: [
      platform === 'android'
        ? androidCapabilities(application)
        : iosCapabilities(application)
    ],
    services: [
      [
        'appium',
        {
          args: {
            address: '127.0.0.1',
            // Appium 3 requires insecure features to be scoped to a driver.
            // chromedriver_autodownload lets the UiAutomator2 driver fetch a
            // chromedriver matching the device's WebView (see E2E_CHROMEDRIVER).
            allowInsecure: 'uiautomator2:chromedriver_autodownload'
          },
          // First-session setup (chromedriver download, WebDriverAgent build)
          // can be slow, hence the generous timeout.
          appiumStartTimeout: 120_000,
          logPath: path.join(dirname, 'logs')
        }
      ]
    ],
    reporters: ['spec'],
    framework: 'mocha',
    mochaOpts: {
      ui: 'bdd',
      timeout: 120000
    },
    connectionRetryCount: 0,
    // The first session boots the simulator/emulator, installs the app and, on
    // iOS, compiles WebDriverAgent — well over the 120s default that the
    // request to create the session would otherwise be cut at (the driver
    // timeouts in the capabilities below are what actually bound it).
    connectionRetryTimeout: 600_000,
    specFileRetries: Number(process.env.E2E_SPEC_RETRIES ?? 0),
    // Tells the specs (which run in worker processes) what the app runs on;
    // `process.platform` there is the host. See `platform` in test/helpers.
    runnerEnv: { E2E_PLATFORM: platform },

    onPrepare: async () => {
      // The example (and the specs' type-checking) resolve the plugins' JS
      // packages from their `dist-js` build output. Fail early with a clear
      // message instead of a confusing vite/tsc error.
      const pluginsBuilt = fs.existsSync(
        path.join(repoRoot, 'plugins', 'fs', 'dist-js', 'index.js')
      )
      if (!pluginsBuilt) {
        throw new Error(
          'the plugins are not built — run `pnpm build` at the repo root before the e2e suite.'
        )
      }

      if (!process.env.E2E_SKIP_BUILD && !process.env.E2E_APP_PATH) {
        // The Android Studio / Xcode projects are committed in this repository
        // (`examples/api/src-tauri/gen`), so this only runs if they were wiped.
        // The build installs the Rust target it needs itself, so init skips that.
        // eslint-disable-next-line security/detect-non-literal-fs-filename
        const projectGenerated = fs.existsSync(
          path.join(tauriDir, 'gen', ios ? 'apple' : 'android')
        )
        if (!projectGenerated) {
          tauriCli([platform, 'init', '--ci', '--skip-targets-install'])
        }
        // `tauri ios build` exports the simulator app with `fs::rename`, which
        // fails with "Directory not empty" when a previous build is still
        // there, so clear it out first.
        // eslint-disable-next-line security/detect-non-literal-fs-filename
        if (ios && fs.existsSync(application)) {
          fs.rmSync(application, { recursive: true, force: true })
        }
        // A debug build, so wry turns on webview debugging (Android
        // `setWebContentsDebuggingEnabled`, iOS `isInspectable`), which is what
        // lets Appium reach the page. Only the target that the device/emulator
        // actually runs is compiled.
        //
        // Unlike the desktop suite this passes no `tauri.e2e.conf.json`
        // override: both things it turns on (the automation plugin and the
        // updater endpoint) belong to desktop-only plugins.
        tauriCli(
          ios
            ? [
                'ios',
                'build',
                '--debug',
                '--target',
                ios.name,
                // Simulator builds are not code signed (see
                // `additionalWebviewBundleIds` in `iosCapabilities`).
                '--no-sign'
              ]
            : [
                'android',
                'build',
                '--debug',
                '--apk',
                '--target',
                androidTarget()
              ],
          ios ? {} : androidBuildEnv()
        )
      }

      // eslint-disable-next-line security/detect-non-literal-fs-filename
      if (!fs.existsSync(application)) {
        throw new Error(
          `app not found at ${application} — build it (unset E2E_SKIP_BUILD) or point E2E_APP_PATH at an existing build.`
        )
      }

      // Serves the upload/download fixtures the `upload` specs hit. Lives in
      // the launcher process so it outlives the per-spec worker processes.
      // (The updater manifest it also serves is only used on desktop, where
      // the updater plugin is registered.)
      fixtureServer = await startFixtureServer()
    },

    // The session starts in the native (`NATIVE_APP`) context. Every spec goes
    // through `window.__TAURI__`, so switch to the app's webview as soon as it
    // is attachable and then block until the page has loaded (as the desktop
    // config does).
    before: async (_capabilities, _specs, browser: WebdriverIO.Browser) => {
      if (platform === 'android') {
        // The fixture server listens on the host's loopback, which on a device
        // (or emulator) is the device's own. `adb reverse` forwards the same
        // port from the device back to the host so `FIXTURE_SERVER_URL` works
        // unchanged in the page. The iOS simulator shares the host's network
        // stack, so it needs nothing. Re-run per spec file (it is idempotent)
        // because Appium may only have booted the emulator with the first
        // session.
        adbReverse(FIXTURE_SERVER_PORT)
      }

      let webview: string | undefined
      await browser.waitUntil(
        async () => {
          const contexts = (await browser.getAppiumContexts())
            // Detailed objects are only returned with `appium:fullContextList`.
            .map((context) =>
              typeof context === 'string' ? context : context.id
            )
          // Android names the context after the package, and lists every
          // debuggable WebView on the device (other apps included), so match
          // ours exactly. iOS names it `WEBVIEW_<pid>.<n>` and only lists the
          // app under test's webviews, so the first one is it.
          webview =
            contexts.find((name) => name === `WEBVIEW_${appId}`)
            ?? (platform === 'ios'
              ? contexts.find((name) => name.startsWith('WEBVIEW_'))
              : undefined)
          return webview !== undefined
        },
        {
          // Covers a cold app start plus, on Android, the on-demand chromedriver
          // download for the first session.
          timeout: 120_000,
          interval: 1000,
          timeoutMsg:
            'no WEBVIEW context appeared — is the app a debug build (webview debugging enabled)?'
        }
      )
      await browser.switchAppiumContext(webview!)
      // The specs run the page through `executeAsync`, and the XCUITest driver
      // starts with a script timeout of 0 (every async script times out at
      // once) rather than the 30s the other drivers default to.
      await browser.setTimeout({ script: 30_000 })

      await browser.waitUntil(
        async () => {
          try {
            const ready: unknown = await browser.executeAsync(
              'var done = arguments[arguments.length - 1]; done(typeof window.__TAURI__ !== "undefined");'
            )
            return ready === true
          } catch {
            // A command issued mid-navigation can fail on a stale execution
            // context; that just means "not ready yet".
            return false
          }
        },
        {
          timeout: 30_000,
          interval: 250,
          timeoutMsg:
            'window.__TAURI__ never became available — the app did not load its page.'
        }
      )
    },

    onComplete: () => {
      closeFixtureServer()
    }
  }
}

function closeFixtureServer(): void {
  fixtureServer?.close()
  fixtureServer = undefined
}

for (const signal of [
  'exit',
  'SIGINT',
  'SIGTERM',
  'SIGHUP',
  'SIGBREAK'
] as const) {
  process.on(signal, () => {
    try {
      closeFixtureServer()
    } finally {
      process.exit()
    }
  })
}

/** Runs `pnpm tauri <args>` in examples/api, failing loudly. */
function tauriCli(args: string[], env: NodeJS.ProcessEnv = {}): void {
  const result = spawnSync('pnpm', ['tauri', ...args], {
    cwd: appDir,
    stdio: 'inherit',
    shell: true,
    env: { ...process.env, ...env }
  })
  if (result.status !== 0) {
    throw new Error(
      `\`pnpm tauri ${args.join(' ')}\` failed with status ${result.status}`
    )
  }
}

// --- Android -----------------------------------------------------------------

/**
 * Environment for the Android build.
 *
 * - No debug info: the suite needs a debug build (webview debugging follows
 *   `debug_assertions`), but with the sql and websocket plugins the
 *   debug info alone grows the APK past what a default emulator can install
 *   ("not enough space").
 */
function androidBuildEnv(): NodeJS.ProcessEnv {
  return { CARGO_PROFILE_DEV_DEBUG: '0' }
}

/** `adb` from the Android SDK, else whatever is on `PATH`. */
function adb(args: string[]): SpawnSyncReturns<string> {
  const sdk = process.env.ANDROID_HOME ?? process.env.ANDROID_SDK_ROOT
  const binary = sdk ? path.join(sdk, 'platform-tools', 'adb') : 'adb'
  return spawnSync(
    binary,
    [
      ...(process.env.E2E_ANDROID_DEVICE
        ? ['-s', process.env.E2E_ANDROID_DEVICE]
        : []),
      ...args
    ],
    { encoding: 'utf8', timeout: 20_000 }
  )
}

function adbReverse(port: number): void {
  const result = adb(['reverse', `tcp:${port}`, `tcp:${port}`])
  if (result.status !== 0) {
    console.warn(
      `\`adb reverse tcp:${port}\` failed — the upload specs will not reach the fixture server.\n${result.stderr ?? ''}`
    )
  }
}

function androidCapabilities(app: string): WebdriverIO.Capabilities {
  return {
    platformName: 'Android',
    'appium:automationName': 'UiAutomator2',
    'appium:app': app,
    'appium:appPackage': appId,
    'appium:appActivity': '.MainActivity',
    // Every build has the same version code, and Appium otherwise skips
    // installing an APK whose version is already on the device, so a rebuilt
    // app would never replace the installed one.
    'appium:enforceAppInstall': true,
    // Which device/emulator to use; Appium picks the first connected one
    // otherwise. E2E_ANDROID_AVD instead boots that AVD.
    ...(process.env.E2E_ANDROID_DEVICE
      ? { 'appium:udid': process.env.E2E_ANDROID_DEVICE }
      : {}),
    ...(process.env.E2E_ANDROID_AVD
      ? { 'appium:avd': process.env.E2E_ANDROID_AVD }
      : {}),
    // chromedriver must match the WebView's Chrome version: either a specific
    // binary, or one Appium downloads on demand.
    ...(process.env.E2E_CHROMEDRIVER
      ? { 'appium:chromedriverExecutable': process.env.E2E_CHROMEDRIVER }
      : { 'appium:chromedriverAutodownload': true }),
    // Grants the runtime permissions the manifest declares (POST_NOTIFICATIONS
    // among them) so the notification specs do not stop on a system dialog.
    'appium:autoGrantPermissions': true,
    // Emulators in CI are slow; give the UiAutomator2 server and adb room.
    'appium:uiautomator2ServerInstallTimeout': 120_000,
    'appium:uiautomator2ServerLaunchTimeout': 120_000,
    'appium:adbExecTimeout': 60_000,
    'appium:newCommandTimeout': 300
  }
}

const androidTargets: Record<string, string> = {
  // `ro.product.cpu.abi` -> `tauri android build --target`
  'arm64-v8a': 'aarch64',
  'armeabi-v7a': 'armv7',
  x86_64: 'x86_64',
  x86: 'i686'
}

/**
 * The Rust target to build the APK for: `E2E_ANDROID_TARGET`, else the ABI of
 * the connected device/emulator (via adb), else the host's (emulators run the
 * host architecture).
 */
function androidTarget(): string {
  if (process.env.E2E_ANDROID_TARGET) {
    return process.env.E2E_ANDROID_TARGET
  }
  const abi = adb(['shell', 'getprop', 'ro.product.cpu.abi'])
  const detected =
    abi.status === 0 ? androidTargets[abi.stdout.trim()] : undefined
  return detected ?? (process.arch === 'arm64' ? 'aarch64' : 'x86_64')
}

// --- iOS ---------------------------------------------------------------------

function iosCapabilities(app: string): WebdriverIO.Capabilities {
  return {
    platformName: 'iOS',
    'appium:automationName': 'XCUITest',
    'appium:app': app,
    'appium:bundleId': appId,
    // The driver looks the app up in the simulator's Web Inspector listing by
    // bundle identifier. The inspector identifies an app by the
    // `application-identifier` entitlement Xcode embeds when it code signs a
    // simulator build (`__TEXT,__entitlements`); `--no-sign` skips signing
    // altogether (`CODE_SIGNING_ALLOWED=NO`), so the entitlement is missing and
    // the inspector falls back to `process-<executable name>`. Match that too.
    'appium:additionalWebviewBundleIds': [
      `process-${path.basename(app, '.app')}`
    ],
    'appium:udid': iosSimulator(),
    // No Simulator.app window. Besides not needing one, the driver otherwise
    // shuts a simulator that is booted without a visible UI down to relaunch it
    // with one, on every session — and that shutdown regularly outlasts the
    // driver's 15s limit on it.
    'appium:isHeadless': true,
    // WebDriverAgent is compiled on the first session, which takes minutes on
    // a CI runner.
    'appium:wdaLaunchTimeout': 240_000,
    'appium:wdaStartupRetries': 3,
    'appium:simulatorStartupTimeout': 240_000,
    'appium:newCommandTimeout': 300
  }
}

/**
 * The Rust target to build the simulator app for (`E2E_IOS_TARGET`, else the
 * host's architecture) and the directory the CLI exports it to.
 */
function iosTarget(): { name: string; outputArch: string } {
  const name =
    process.env.E2E_IOS_TARGET
    ?? (process.arch === 'arm64' ? 'aarch64-sim' : 'x86_64')
  // `tauri ios build` writes to gen/apple/build/<arch>/ (cargo-mobile2's
  // `arch` for the target).
  const outputArch = name === 'aarch64-sim' ? 'arm64-sim' : name
  return { name, outputArch }
}

interface SimctlDevice {
  name: string
  udid: string
  state: string
  isAvailable: boolean
  deviceTypeIdentifier?: string
}

/**
 * UDID of the simulator to run on: `E2E_IOS_DEVICE` (a UDID or device name),
 * else an already-booted iPhone, else the iPhone on the newest installed
 * runtime. Resolved through `simctl` so nothing has to be hardcoded per Xcode
 * version.
 */
function iosSimulator(): string {
  const requested = process.env.E2E_IOS_DEVICE
  if (requested && /^[0-9A-F-]{36}$/i.test(requested)) {
    return requested
  }
  const list = spawnSync(
    'xcrun',
    ['simctl', 'list', 'devices', 'available', '--json'],
    { encoding: 'utf8' }
  )
  if (list.status !== 0) {
    throw new Error(
      `\`xcrun simctl list\` failed — is Xcode installed?\n${list.stderr}`
    )
  }
  const { devices: runtimes } = JSON.parse(list.stdout) as {
    devices: Record<string, SimctlDevice[]>
  }
  const iphones = Object.entries(runtimes)
    // e.g. `com.apple.CoreSimulator.SimRuntime.iOS-18-2`
    .filter(([runtime]) => runtime.includes('.iOS-'))
    .sort(([a], [b]) => runtimeVersion(b) - runtimeVersion(a))
    .flatMap(([, devices]) =>
      devices.filter(
        (device) =>
          device.isAvailable
          && (device.deviceTypeIdentifier ?? device.name).includes('iPhone')
      )
    )
  const device = requested
    ? iphones.find((candidate) => candidate.name === requested)
    : (iphones.find((candidate) => candidate.state === 'Booted') ?? iphones[0])
  if (!device) {
    throw new Error(
      requested
        ? `no available iPhone simulator named "${requested}" (E2E_IOS_DEVICE)`
        : 'no available iPhone simulator found — install an iOS runtime in Xcode.'
    )
  }
  return device.udid
}

function runtimeVersion(runtime: string): number {
  const [major = '0', minor = '0'] = (runtime.split('.iOS-')[1] ?? '').split(
    '-'
  )
  return Number(major) * 100 + Number(minor)
}
