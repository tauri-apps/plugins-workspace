// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, type PluginApi } from '../helpers/index.js'

/**
 * The members each plugin's `api-iife.js` is expected to define on
 * `window.__TAURI__.<plugin>`. This is the one place the suite covers the
 * plugins whose commands cannot be driven from a WebDriver session (dialog
 * blocks on native UI, process terminates the app), and it catches a plugin
 * whose global API script is missing from its `build.rs`.
 */
const surface: { [P in keyof PluginApi]: (keyof PluginApi[P])[] } = {
  cli: ['getMatches'],
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
  globalShortcut: ['register', 'unregister', 'unregisterAll', 'isRegistered'],
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
  updater: ['check', 'Update'],
  upload: ['download', 'upload', 'HttpMethod'],
  windowState: [
    'StateFlags',
    'restoreState',
    'restoreStateCurrent',
    'saveWindowState',
    'filename'
  ]
}

describe('plugin globals', () => {
  it('every desktop plugin registers its API on window.__TAURI__', async () => {
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
        members as string[]
      )
      expect(missing).toEqual([])
    })
  }
})
