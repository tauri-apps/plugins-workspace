// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { execFile, spawnSync } from 'node:child_process'
import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  eventually,
  itOn,
  platform
} from '../helpers/index.js'
import { adbArgs, simctlArgs } from '../helpers/device.js'

// The location is simulated from the host: `adb emu geo fix` on the Android
// emulator and `simctl location` on the iOS simulator. Neither prompts for
// permission: Appium grants the manifest's location permissions on Android
// (`appium:autoGrantPermissions`), and on iOS the suite grants it through
// `simctl privacy`. Real Android devices cannot be driven this way, so the
// suite expects an emulator there.

const appId = 'com.tauri.api'

interface Point {
  latitude: number
  longitude: number
}

const EIFFEL_TOWER: Point = { latitude: 48.8584, longitude: 2.2945 }
const TOKYO_TOWER: Point = { latitude: 35.6586, longitude: 139.7454 }
const SUGARLOAF: Point = { latitude: -22.9486, longitude: -43.1566 }
const OPERA_HOUSE: Point = { latitude: -33.8568, longitude: 151.2153 }

const OPTIONS = { enableHighAccuracy: true, timeout: 1000, maximumAge: 0 }

function adb(args: string[]): void {
  const [binary, fullArgs] = adbArgs(args)
  const result = spawnSync(binary, fullArgs, {
    encoding: 'utf8',
    timeout: 20_000
  })
  if (result.status !== 0) {
    throw new Error(`adb ${args.join(' ')} failed: ${result.stderr}`)
  }
}

/** The command that moves the simulated device to `point`. */
function locationCommand(point: Point): [string, string[]] {
  if (platform === 'android') {
    // Longitude first.
    return adbArgs([
      'emu',
      'geo',
      'fix',
      String(point.longitude),
      String(point.latitude)
    ])
  }
  return simctlArgs(['location', 'set', `${point.latitude},${point.longitude}`])
}

/**
 * Grants the app location access on the simulator. locationd forgets an app's
 * authorization whenever it is reinstalled, which Appium does at the start of
 * every session, so this has to run within the session. It does not terminate
 * the running app, and the plugin reads the new status on its next call.
 */
function grantIosLocation(): void {
  const [binary, args] = simctlArgs(['privacy', 'grant', 'location', appId])
  const result = spawnSync(binary, args, { encoding: 'utf8', timeout: 20_000 })
  if (result.status !== 0) {
    throw new Error(`simctl privacy grant failed: ${result.stderr}`)
  }
}

/**
 * Moves the simulated device to `point`, repeating every second until the
 * returned function is called. The emulator only reports a `geo fix` to
 * requests that are already listening, so a single fix sent before the app
 * asks would be missed.
 */
function holdLocation(point: Point): () => void {
  const [binary, args] = locationCommand(point)
  // The first one synchronously, so the device is there before the caller
  // asks for a position.
  spawnSync(binary, args, { timeout: 20_000 })
  const timer = setInterval(() => execFile(binary, args, () => {}), 1000)
  return () => clearInterval(timer)
}

/** Runs `fn` while the device sits at `point`. */
async function at<T>(point: Point, fn: () => Promise<T>): Promise<T> {
  const release = holdLocation(point)
  try {
    return await fn()
  } finally {
    release()
  }
}

function expectAt(position: Point | undefined, point: Point): void {
  expect(position).toBeDefined()
  expect(position!.latitude).toBeCloseTo(point.latitude, 3)
  expect(position!.longitude).toBeCloseTo(point.longitude, 3)
}

/** Starts a watch that records into `window.__geolocationE2E[key]`. */
function startWatch(key: string): Promise<number> {
  return tauri(
    async (api, key, options) => {
      const w = window as unknown as {
        __geolocationE2E?: Record<
          string,
          {
            positions: { latitude: number; longitude: number }[]
            errors: string[]
          }
        >
      }
      w.__geolocationE2E ??= {}
      const log = { positions: [], errors: [] } as {
        positions: { latitude: number; longitude: number }[]
        errors: string[]
      }
      // eslint-disable-next-line security/detect-object-injection
      w.__geolocationE2E[key] = log
      return api.geolocation.watchPosition(options, (position, error) => {
        if (position) {
          log.positions.push({
            latitude: position.coords.latitude,
            longitude: position.coords.longitude
          })
        } else {
          log.errors.push(String(error))
        }
      })
    },
    key,
    OPTIONS
  )
}

/** What the watch started with `key` has recorded so far. */
function watchLog(
  key: string
): Promise<{ positions: Point[]; errors: string[] }> {
  return tauri((_api, key) => {
    const w = window as unknown as {
      __geolocationE2E: Record<
        string,
        {
          positions: { latitude: number; longitude: number }[]
          errors: string[]
        }
      >
    }
    // eslint-disable-next-line security/detect-object-injection
    return w.__geolocationE2E[key]
  }, key)
}

/** Waits until the watch's latest position is `point`. */
async function watchReaches(key: string, point: Point): Promise<void> {
  await eventually(
    async () => {
      const log = await watchLog(key)
      expectAt(log.positions.at(-1), point)
    },
    { timeout: 30_000, interval: 500 }
  )
}

describePlugin('geolocation', { mobileOnly: true }, () => {
  const watches: number[] = []

  before(() => {
    if (platform === 'ios') {
      grantIosLocation()
    }
  })

  afterEach(async () => {
    for (const id of watches.splice(0)) {
      await tauri((api, id) => api.geolocation.clearWatch(id), id)
    }
  })

  it('reports the location permission as granted', async () => {
    const status = await tauri((api) => api.geolocation.checkPermissions())
    expect(status.location).toBe('granted')
    expect(status.coarseLocation).toBe('granted')
    // Already granted, so this answers without a prompt.
    const requested = await tauri((api) =>
      api.geolocation.requestPermissions(['location'])
    )
    expect(requested.location).toBe('granted')
  })

  it('returns the current position', async () => {
    const position = await at(EIFFEL_TOWER, () =>
      tauri((api, options) => api.geolocation.getCurrentPosition(options), {
        ...OPTIONS,
        timeout: 20_000
      })
    )
    expectAt(position.coords, EIFFEL_TOWER)
    expect(typeof position.timestamp).toBe('number')
    expect(typeof position.coords.accuracy).toBe('number')
  })

  it('reports position changes to a watch', async () => {
    watches.push(await startWatch('single'))
    await at(TOKYO_TOWER, () => watchReaches('single', TOKYO_TOWER))
    await at(SUGARLOAF, () => watchReaches('single', SUGARLOAF))
    // Core Location reports `kCLErrorLocationUnknown` (error 0) when it cannot
    // get a fix right away and keeps trying; the plugin forwards it to the
    // watch as is. Only that transient error is tolerated.
    const errors = (await watchLog('single')).errors.filter(
      (error) => !error.includes('kCLErrorDomain error 0.')
    )
    expect(errors).toEqual([])
  })

  it('keeps several watches active at once', async () => {
    watches.push(await startWatch('first'))
    await at(TOKYO_TOWER, () => watchReaches('first', TOKYO_TOWER))
    // Starting another watch must not cancel the first one.
    watches.push(await startWatch('second'))
    await at(OPERA_HOUSE, async () => {
      await watchReaches('second', OPERA_HOUSE)
      await watchReaches('first', OPERA_HOUSE)
    })
  })

  it('stops reporting after clearWatch', async () => {
    const id = await startWatch('cleared')
    await at(EIFFEL_TOWER, () => watchReaches('cleared', EIFFEL_TOWER))
    await tauri((api, id) => api.geolocation.clearWatch(id), id)
    const before = (await watchLog('cleared')).positions.length
    await at(
      SUGARLOAF,
      () => new Promise((resolve) => setTimeout(resolve, 5000))
    )
    const after = await watchLog('cleared')
    expect(after.positions.length).toBe(before)
    expect(
      after.positions.some(
        (p) => Math.abs(p.latitude - SUGARLOAF.latitude) < 0.001
      )
    ).toBe(false)
  })

  // Location services can only be toggled from the host on Android.
  itOn('android', 'rejects while location services are off', async () => {
    adb(['shell', 'cmd', 'location', 'set-location-enabled', 'false'])
    try {
      await eventually(async () => {
        expect(
          await tauriError((api) => api.geolocation.checkPermissions())
        ).toContain('Location services are disabled')
      })
      expect(
        await tauriError(
          (api, options) => api.geolocation.getCurrentPosition(options),
          OPTIONS
        )
      ).toContain('Location disabled')
    } finally {
      adb(['shell', 'cmd', 'location', 'set-location-enabled', 'true'])
    }
  })
})
