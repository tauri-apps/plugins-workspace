// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import path from 'node:path'
import { spawnSync } from 'node:child_process'
import { expect } from '@wdio/globals'
import { tauri, describePlugin, platform } from '../helpers/index.js'

// A successful authentication needs a finger or a face, which the driver cannot
// provide, so only the paths that reject without showing the prompt are covered.

/**
 * Whether the Android device has no lock screen credential (the default on a
 * fresh emulator). `locksettings verify` succeeds without a credential only then.
 */
function androidHasNoLockCredential(): boolean {
  const sdk = process.env.ANDROID_HOME ?? process.env.ANDROID_SDK_ROOT
  const adb = sdk ? path.join(sdk, 'platform-tools', 'adb') : 'adb'
  const device = process.env.E2E_ANDROID_DEVICE
    ? ['-s', process.env.E2E_ANDROID_DEVICE]
    : []
  const result = spawnSync(
    adb,
    [...device, 'shell', 'locksettings', 'verify'],
    { encoding: 'utf8', timeout: 20_000 }
  )
  return result.status === 0 && /verified successfully/.test(result.stdout)
}

describePlugin('biometric', { mobileOnly: true }, () => {
  // Android only, where the lock screen state can be read over adb.
  const itNoLockCredential =
    platform === 'android' && androidHasNoLockCredential() ? it : it.skip

  itNoLockCredential(
    'rejects with passcodeNotSet when the fallback is allowed and no credential is set',
    async () => {
      const code = await tauri(async (api) => {
        try {
          await api.biometric.authenticate('e2e', {
            allowDeviceCredential: true
          })
          return 'resolved'
        } catch (error) {
          return (error as { code?: string }).code ?? null
        }
      })
      expect(code).toBe('passcodeNotSet')
    }
  )
})
