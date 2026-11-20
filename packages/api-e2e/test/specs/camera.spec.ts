// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { spawnSync } from 'node:child_process'
import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  eventually,
  itOn,
  platform
} from '../helpers/index.js'
import { adbArgs } from '../helpers/device.js'

// The driver cannot take a picture, and the gallery pickers are other apps'
// UI that differs between OS versions, so no image is ever returned here.
// Covered instead: the permission state, the options that are rejected before
// any UI shows, and (Android) dismissing the system photo picker with the back
// key, which the plugin reports as a cancellation.

const appId = 'com.tauri.api'
const PERMISSION_STATES = [
  'granted',
  'denied',
  'prompt',
  'prompt-with-rationale'
]

function adb(args: string[]): string {
  const [binary, fullArgs] = adbArgs(args)
  const result = spawnSync(binary, fullArgs, {
    encoding: 'utf8',
    timeout: 20_000
  })
  if (result.status !== 0) {
    throw new Error(`adb ${args.join(' ')} failed: ${result.stderr}`)
  }
  return result.stdout
}

/** Whether the example app's window has the input focus on the Android device. */
function appHasFocus(): boolean {
  const focus = /mCurrentFocus=.*/.exec(adb(['shell', 'dumpsys', 'window']))
  return focus !== null && focus[0].includes(`${appId}/`)
}

type Outcome = { ok: true; value: unknown } | { ok: false; message: string }

interface Page {
  __e2eCamera?: Outcome
}

/**
 * Starts a call that opens the photo picker without waiting for it, since it
 * only settles once the picker closes. See {@link outcome}.
 */
async function start(command: 'getPhoto' | 'pickImages'): Promise<void> {
  await tauri((api, command) => {
    const page = window as unknown as Page
    page.__e2eCamera = undefined
    const call =
      command === 'getPhoto'
        ? api.camera.getPhoto({ source: api.camera.Source.Photos })
        : api.camera.pickImages({ limit: 2 })
    call.then(
      (value) => {
        page.__e2eCamera = { ok: true, value }
      },
      (error: unknown) => {
        page.__e2eCamera = { ok: false, message: String(error) }
      }
    )
  }, command)
}

/** Waits for the call made by {@link start} to settle. */
async function outcome(): Promise<Outcome> {
  return await eventually(
    async () => {
      const outcome = await tauri(
        () => (window as unknown as Page).__e2eCamera ?? null
      )
      if (!outcome) {
        throw new Error('the camera call did not settle')
      }
      return outcome
    },
    { timeout: 30_000, interval: 500 }
  )
}

/** Closes the photo picker with the back key once it is in front of the app. */
async function dismissPicker(): Promise<void> {
  await eventually(
    () => {
      if (appHasFocus()) {
        throw new Error('the photo picker did not open')
      }
    },
    { timeout: 30_000, interval: 500 }
  )
  // A key press while the picker is still animating in can be dropped, so
  // repeat it until the app is back. It is never sent while the app has the
  // focus, where it would navigate the webview instead.
  await eventually(
    () => {
      if (!appHasFocus()) {
        adb(['shell', 'input', 'keyevent', 'KEYCODE_BACK'])
        throw new Error('the photo picker did not close')
      }
    },
    { timeout: 30_000, interval: 1500 }
  )
}

describePlugin('camera', { mobileOnly: true }, () => {
  it('reports the permission state', async () => {
    const status = await tauri((api) => api.camera.checkPermissions())
    expect(PERMISSION_STATES).toContain(status.camera)
    expect(PERMISSION_STATES).toContain(status.photos)
    if (platform === 'android') {
      // The example does not declare the CAMERA permission, so the camera
      // app can be used without it. Saving to the gallery only needs one on
      // Android 9 and older, where Appium grants it at install.
      expect(status).toEqual({ camera: 'granted', photos: 'granted' })
    }
  })

  // Nothing is left to ask for on Android (see above), so this answers
  // without a prompt. On iOS it would show one.
  itOn('android', 'requests permissions that are already granted', async () => {
    expect(await tauri((api) => api.camera.requestPermissions())).toEqual({
      camera: 'granted',
      photos: 'granted'
    })
    expect(
      await tauri((api) => api.camera.requestPermissions(['camera']))
    ).toEqual({ camera: 'granted', photos: 'granted' })
  })

  itOn('android', 'rejects an unknown source', async () => {
    expect(
      await tauriError((api) =>
        api.camera.getPhoto({ source: 'bogus' as never })
      )
    ).toContain('Invalid source option')
  })

  itOn('android', 'rejects an unknown result type', async () => {
    expect(
      await tauriError((api) =>
        api.camera.getPhoto({
          source: api.camera.Source.Photos,
          resultType: 'bogus' as never
        })
      )
    ).toContain('Invalid resultType option')
  })

  itOn(
    'android',
    'getPhoto resolves to null when the picker is dismissed',
    async () => {
      await start('getPhoto')
      await dismissPicker()
      expect(await outcome()).toEqual({ ok: true, value: null })
    }
  )

  itOn(
    'android',
    'pickImages resolves to no images when the picker is dismissed',
    async () => {
      await start('pickImages')
      await dismissPicker()
      expect(await outcome()).toEqual({ ok: true, value: [] })
    }
  )
})
