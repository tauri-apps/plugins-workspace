// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Host-side commands that drive the emulator/simulator under test directly,
// for what the app cannot do to itself (simulate a location, open a URL
// through the OS, ...).

import path from 'node:path'

/** `adb` from the Android SDK (else `PATH`), on `E2E_ANDROID_DEVICE` if set. */
export function adbArgs(args: string[]): [string, string[]] {
  const sdk = process.env.ANDROID_HOME ?? process.env.ANDROID_SDK_ROOT
  const binary = sdk ? path.join(sdk, 'platform-tools', 'adb') : 'adb'
  const device = process.env.E2E_ANDROID_DEVICE
    ? ['-s', process.env.E2E_ANDROID_DEVICE]
    : []
  return [binary, [...device, ...args]]
}

/** `xcrun simctl <command> <simulator> <args>` on the simulator under test. */
export function simctlArgs([command, ...args]: string[]): [string, string[]] {
  const simulator = process.env.E2E_IOS_UDID
  if (!simulator) {
    throw new Error(
      'E2E_IOS_UDID is not set — run the suite via wdio.ios.conf.ts'
    )
  }
  return ['xcrun', ['simctl', command, simulator, ...args]]
}
