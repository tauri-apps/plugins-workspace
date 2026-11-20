// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { spawnSync } from 'node:child_process'
import { browser, expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  eventually,
  itOn,
  platform,
  type Platform
} from '../helpers/index.js'
import { adbArgs, simctlArgs } from '../helpers/device.js'

// The suite launches the app without a URL, so there is no current deep link.
// On desktop it cannot open one through the OS, so `onOpenUrl` is exercised by
// emitting the event the plugin itself emits when the app is opened with one.
//
// On mobile the URL is opened through the OS from the host (`am start` on
// Android, `simctl openurl` on iOS), which only reaches the app if the build
// script registered its scheme in the AndroidManifest.xml / Info.plist from the
// example's `plugins.deep-link.mobile` config.
//
// Registering a scheme at runtime is only implemented on Linux (a `.desktop`
// handler plus `xdg-mime`) and Windows (the registry); macOS and mobile read
// the schemes from the bundle and report the runtime APIs as unsupported.

/** A scheme no real app handles, so registering it cannot clobber anything. */
const scheme = 'tauri-plugins-e2e'

/**
 * Configured next to `https` in one entry of the example's
 * `plugins.deep-link.mobile`, which iOS must still register as a URL scheme.
 */
const mixedScheme = 'tauri-plugins-e2e-mixed'

const appId = 'com.tauri.api'

/**
 * Opens `url` through the OS, as another app would. Resolves with whether the
 * OS found an app to hand it to.
 */
function openThroughOs(url: string): { opened: boolean; output: string } {
  const [binary, args] =
    platform === 'android'
      ? adbArgs([
          'shell',
          'am',
          'start',
          '-W',
          '-a',
          'android.intent.action.VIEW',
          // `adb shell` runs the command through the device's shell.
          '-d',
          `'${url}'`,
          appId
        ])
      : simctlArgs(['openurl', url])
  const result = spawnSync(binary, args, { encoding: 'utf8', timeout: 30_000 })
  const output = `${result.stdout ?? ''}${result.stderr ?? ''}`
  // `am start` exits with 0 even when no activity matches the intent.
  const opened = result.status === 0 && !/^Error/m.test(output)
  return { opened, output }
}

/**
 * Answers the "Open in “Tauri API”?" confirmation iOS shows before it hands a
 * custom-scheme URL from outside the app (here `simctl openurl`) to the app.
 * The alert belongs to SpringBoard, which WebDriverAgent only searches from the
 * native context once it is made the active application.
 */
async function acceptIosOpenPrompt(): Promise<void> {
  const webview = (await browser.getAppiumContext()) as string
  await browser.switchAppiumContext('NATIVE_APP')
  await browser.updateSettings({
    defaultActiveApplication: 'com.apple.springboard'
  })
  try {
    const open = browser.$(
      '-ios predicate string:type == "XCUIElementTypeButton" AND label == "Open"'
    )
    // Not shown again once the user picked "Open" for the same source.
    const shown = await open
      .waitForExist({ timeout: 5_000 })
      .then(() => true)
      .catch(() => false)
    if (shown) {
      await open.click()
    }
  } finally {
    await browser.updateSettings({ defaultActiveApplication: 'auto' })
    await browser.switchAppiumContext(webview)
  }
}

/** Records every `onOpenUrl` delivery into `window.__deepLinkE2E`. */
async function recordOpenedUrls(): Promise<void> {
  await tauri(async (api) => {
    const w = window as unknown as { __deepLinkE2E?: string[][] }
    if (w.__deepLinkE2E) {
      return
    }
    w.__deepLinkE2E = []
    await api.deepLink.onOpenUrl((urls) => w.__deepLinkE2E!.push(urls))
  })
}

async function openedUrls(): Promise<string[][]> {
  return tauri(
    () =>
      (window as unknown as { __deepLinkE2E?: string[][] }).__deepLinkE2E ?? []
  )
}

describePlugin('deep-link', () => {
  it('getCurrent is null when the app was not opened with a URL', async () => {
    expect(await tauri((api) => api.deepLink.getCurrent())).toBeNull()
  })

  it('onOpenUrl delivers the opened URLs until unlistened', async () => {
    const received = await tauri(
      async (api, urls) => {
        const received: string[][] = []
        const unlisten = await api.deepLink.onOpenUrl((urls) =>
          received.push(urls)
        )
        await api.event.emit('deep-link://new-url', urls)
        await new Promise((resolve) => setTimeout(resolve, 500))
        unlisten()
        await api.event.emit('deep-link://new-url', ['ignored://'])
        await new Promise((resolve) => setTimeout(resolve, 500))
        return received
      },
      [`${scheme}://open/path?query=1`]
    )
    expect(received).toEqual([[`${scheme}://open/path?query=1`]])
  })

  itOn(
    ['linux', 'win32'],
    'register and unregister toggle the scheme handler',
    async () => {
      const result = await tauri(async (api, scheme) => {
        await api.deepLink.register(scheme)
        const registered = await api.deepLink.isRegistered(scheme)
        await api.deepLink.unregister(scheme)
        return {
          registered,
          afterUnregister: await api.deepLink.isRegistered(scheme)
        }
      }, scheme)
      expect(result).toEqual({ registered: true, afterUnregister: false })
    }
  )

  itOn(
    ['linux', 'win32'],
    'isRegistered is false for an unknown scheme',
    async () => {
      expect(
        await tauri(
          (api, scheme) => api.deepLink.isRegistered(scheme),
          `${scheme}-unknown`
        )
      ).toBe(false)
    }
  )

  itOn(
    ['darwin', 'android', 'ios'],
    'runtime registration is unsupported',
    async () => {
      for (const command of [
        'register',
        'unregister',
        'isRegistered'
      ] as const) {
        const error = await tauriError(
          // eslint-disable-next-line security/detect-object-injection
          (api, command, scheme) => api.deepLink[command](scheme),
          command,
          scheme
        )
        expect(error).toMatch(/unsupported platform/i)
      }
    }
  )

  for (const { url, platforms } of [
    { url: `${scheme}://open/path?query=1`, platforms: ['android', 'ios'] },
    {
      url: `${mixedScheme}://e2e.tauri.app/path`,
      platforms: ['android', 'ios']
    },
    // iOS only routes https URLs to an app through universal links, which need
    // the associated domains entitlement of a signed build (the suite's
    // simulator build is unsigned) and a verified apple-app-site-association.
    // `am start` with the package skips Android's App Links verification but
    // still has to match the https intent filter.
    { url: 'https://e2e.tauri.app/path', platforms: ['android'] }
  ] satisfies { url: string; platforms: Platform[] }[]) {
    itOn(
      platforms,
      `delivers ${url.split('://')[0]}:// URLs opened through the OS`,
      async () => {
        await recordOpenedUrls()
        const before = (await openedUrls()).length

        const { opened, output } = openThroughOs(url)
        expect({ opened, output }).toEqual({ opened: true, output })
        if (platform === 'ios') {
          await acceptIosOpenPrompt()
        }

        await eventually(
          async () => {
            expect((await openedUrls()).slice(before)).toEqual([[url]])
            expect(await tauri((api) => api.deepLink.getCurrent())).toEqual([
              url
            ])
          },
          { timeout: 15_000 }
        )
      }
    )
  }

  // Guards the test above: if the OS handed every URL to the app, it would
  // pass without any scheme being registered.
  itOn(
    ['android', 'ios'],
    'the OS does not route an unregistered scheme to the app',
    () => {
      const { opened } = openThroughOs(`${scheme}-unregistered://open`)
      expect(opened).toBe(false)
    }
  )
})
