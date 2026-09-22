// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import {
  tauri,
  isMobile,
  type CommonPluginApi,
  type DesktopPluginApi,
  type MobilePluginApi
} from '../helpers/index.js'

/**
 * The members each plugin's `api-iife.js` is expected to define on
 * `window.__TAURI__.<plugin>`. This is the one place the suite covers the
 * plugins whose commands cannot be driven from a WebDriver session (dialog
 * blocks on native UI, process terminates the app, the mobile plugins need
 * hardware or native UI), and it catches a plugin whose global API script is
 * missing from its `build.rs`.
 */
type Surface<T> = { [P in keyof T]: (keyof T[P])[] }

/** Plugins the example registers on every platform. */
const commonSurface: Surface<CommonPluginApi> = {
  clipboardManager: [
    'writeText',
    'readText',
    'writeHtml',
    'clear',
    'readImage',
    'writeImage'
  ],
  dialog: ['open', 'save', 'message', 'ask', 'confirm'],
  fs: [
    'BaseDirectory',
    'FileHandle',
    'SeekMode',
    'create',
    'open',
    'copyFile',
    'mkdir',
    'readDir',
    'readFile',
    'readTextFile',
    'readTextFileLines',
    'remove',
    'rename',
    'stat',
    'lstat',
    'truncate',
    'writeFile',
    'writeTextFile',
    'exists',
    'watch',
    'watchImmediate',
    'size'
  ],
  http: ['fetch'],
  log: [
    'LogLevel',
    'error',
    'warn',
    'info',
    'debug',
    'trace',
    'attachLogger',
    'attachConsole'
  ],
  notification: [
    'isPermissionGranted',
    'requestPermission',
    'sendNotification',
    'registerActionTypes',
    'pending',
    'cancel',
    'cancelAll',
    'active',
    'removeActive',
    'removeAllActive',
    'createChannel',
    'removeChannel',
    'channels',
    'onNotificationReceived',
    'onAction'
  ],
  opener: ['openUrl', 'openPath', 'revealItemInDir'],
  os: [
    'eol',
    'platform',
    'family',
    'version',
    'type',
    'arch',
    'locale',
    'exeExtension',
    'hostname'
  ],
  process: ['exit', 'relaunch'],
  shell: ['Command', 'Child', 'EventEmitter', 'open'],
  store: ['load', 'getStore', 'LazyStore', 'Store'],
  upload: ['download', 'upload', 'HttpMethod']
}

/** Plugins the example only registers on desktop. */
const desktopSurface: Surface<DesktopPluginApi> = {
  cli: ['getMatches'],
  globalShortcut: ['register', 'unregister', 'unregisterAll', 'isRegistered'],
  updater: ['check', 'Update'],
  windowState: [
    'StateFlags',
    'restoreState',
    'restoreStateCurrent',
    'saveWindowState',
    'filename'
  ]
}

/** Plugins the example only registers on mobile. */
const mobileSurface: Surface<MobilePluginApi> = {
  barcodeScanner: [
    'Format',
    'scan',
    'cancel',
    'checkPermissions',
    'requestPermissions',
    'openAppSettings'
  ],
  biometric: ['BiometryType', 'checkStatus', 'authenticate'],
  geolocation: [
    'watchPosition',
    'getCurrentPosition',
    'clearWatch',
    'checkPermissions',
    'requestPermissions'
  ],
  // `ImpactFeedbackStyle` and `NotificationFeedbackType` are type aliases, so
  // they are not part of the runtime namespace.
  haptics: [
    'vibrate',
    'impactFeedback',
    'notificationFeedback',
    'selectionFeedback'
  ],
  nfc: [
    'NFCTypeNameFormat',
    'TechKind',
    'RTD_TEXT',
    'RTD_URI',
    'record',
    'textRecord',
    'uriRecord',
    'scan',
    'write',
    'isAvailable'
  ]
}

const surface = {
  ...commonSurface,
  ...(isMobile ? mobileSurface : desktopSurface)
} as Record<string, string[]>

/** The other platform's plugins, which must *not* be in this build. */
const foreign = Object.keys(isMobile ? desktopSurface : mobileSurface)

describe('plugin globals', () => {
  it('every plugin this platform registers exposes its API on window.__TAURI__', async () => {
    const missing = await tauri(
      (api, plugins) =>
        plugins.filter(
          (plugin) =>
            // eslint-disable-next-line security/detect-object-injection
            typeof (api as unknown as Record<string, unknown>)[plugin]
            !== 'object'
        ),
      Object.keys(surface)
    )
    expect(missing).toEqual([])
  })

  it('the other platform’s plugins are not in the build', async () => {
    // Their Rust crates are target-gated in the example's Cargo.toml, so their
    // `global_api_script_path` is never injected either.
    const present = await tauri(
      (api, plugins) =>
        plugins.filter(
          (plugin) =>
            // eslint-disable-next-line security/detect-object-injection
            (api as unknown as Record<string, unknown>)[plugin] !== undefined
        ),
      foreign
    )
    expect(present).toEqual([])
  })

  for (const [plugin, members] of Object.entries(surface)) {
    it(`${plugin} exposes its documented members`, async () => {
      const missing = await tauri(
        (api, plugin, members) => {
          const namespaces = api as unknown as Record<
            string,
            Record<string, unknown>
          >
          // eslint-disable-next-line security/detect-object-injection
          const namespace = namespaces[plugin]
          return members.filter((member) => !(member in namespace))
        },
        plugin,
        members
      )
      expect(missing).toEqual([])
    })
  }
})
