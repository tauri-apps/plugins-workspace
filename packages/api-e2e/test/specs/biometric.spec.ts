// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import path from 'node:path'
import { spawnSync, type SpawnSyncReturns } from 'node:child_process'
import { browser, expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  eventually,
  platform
} from '../helpers/index.js'

// The driver cannot provide a finger or a face, so `authenticate` is only
// covered on the paths that reject without showing the prompt. On an Android
// emulator a fingerprint can be enrolled and touched over `adb emu`, which is
// what the `biometricCipher` specs below rely on.

const appId = 'com.tauri.api'
/** Lock screen PIN set for the fingerprint enrollment, cleared afterwards. */
const PIN = '1234'

/** `adb` from the Android SDK (else `PATH`), on `E2E_ANDROID_DEVICE` if set. */
function adb(args: string[]): SpawnSyncReturns<string> {
  const sdk = process.env.ANDROID_HOME ?? process.env.ANDROID_SDK_ROOT
  const binary = sdk ? path.join(sdk, 'platform-tools', 'adb') : 'adb'
  const device = process.env.E2E_ANDROID_DEVICE
    ? ['-s', process.env.E2E_ANDROID_DEVICE]
    : []
  return spawnSync(binary, [...device, ...args], {
    encoding: 'utf8',
    timeout: 20_000
  })
}

/**
 * Whether the Android device has no lock screen credential (the default on a
 * fresh emulator). `locksettings verify` succeeds without a credential only then.
 */
function androidHasNoLockCredential(): boolean {
  const result = adb(['shell', 'locksettings', 'verify'])
  return result.status === 0 && /verified successfully/.test(result.stdout)
}

/** Whether the Android device is an emulator, whose fingerprint sensor `adb emu` drives. */
function androidIsEmulator(): boolean {
  const result = adb(['shell', 'getprop', 'ro.kernel.qemu'])
  const legacy = result.stdout?.trim() === '1'
  return (
    legacy || adb(['shell', 'getprop', 'ro.boot.qemu']).stdout?.trim() === '1'
  )
}

/** Puts finger 1 on the emulator's sensor and lifts it again. */
function touchFingerprintSensor(): void {
  adb(['emu', 'finger', 'touch', '1'])
  adb(['emu', 'finger', 'remove', '1'])
}

/**
 * Sets the lock screen PIN and enrolls finger 1 through the Settings
 * enrollment flow, driven from the native context. The screens differ between
 * Android versions, so this answers whichever one is showing until the flow
 * reports the fingerprint as added.
 */
async function enrollFingerprint(): Promise<void> {
  const pin = adb(['shell', 'locksettings', 'set-pin', PIN])
  if (pin.status !== 0) {
    throw new Error(`locksettings set-pin failed: ${pin.stderr || pin.stdout}`)
  }
  adb(['shell', 'am', 'start', '-a', 'android.settings.FINGERPRINT_ENROLL'])

  await browser.switchAppiumContext('NATIVE_APP')
  const pinEntry = browser.$('id=com.android.settings:id/password_entry')
  const done = browser.$(
    'android=new UiSelector().clickable(true).textMatches("(?i)^done$")'
  )
  // Intro and education screens. "More" scrolls the consent text until it
  // turns into "I agree".
  const proceed = browser.$(
    'android=new UiSelector().clickable(true).textMatches("(?i)^(i agree|more|next|start|continue|ok)$")'
  )

  await browser.waitUntil(
    async () => {
      if (await done.isExisting()) {
        await done.click()
        return true
      }
      if (await pinEntry.isExisting()) {
        await pinEntry.setValue(PIN)
        await browser.pressKeyCode(66 /* KEYCODE_ENTER */)
      } else if (await proceed.isExisting()) {
        await proceed.click()
      } else {
        // "Touch the sensor" and the enrollment progress screens.
        touchFingerprintSensor()
      }
      return false
    },
    {
      timeout: 120_000,
      interval: 1000,
      timeoutMsg: 'the fingerprint enrollment flow did not complete'
    }
  )
}

/** Brings the app back to the foreground and returns to its webview. */
async function returnToApp(): Promise<void> {
  await browser.activateApp(appId)
  await switchToWebview()
}

/** Switches back to the app's webview once it is attachable again. */
async function switchToWebview(): Promise<void> {
  await browser.waitUntil(
    async () => {
      try {
        await browser.switchAppiumContext(`WEBVIEW_${appId}`)
        return true
      } catch {
        return false
      }
    },
    {
      timeout: 30_000,
      interval: 1000,
      timeoutMsg: 'the webview did not come back'
    }
  )
}

type CipherOptions = { dataToEncrypt: string } | { dataToDecrypt: string }

type CipherOutcome =
  | { ok: true; data: string }
  | { ok: false; code: string | null; message: string }

/**
 * Starts a `biometricCipher` call without waiting for it, since it only
 * settles once the prompt is answered. See {@link cipherOutcome}.
 */
async function startCipher(options: CipherOptions): Promise<void> {
  await tauri((api, options) => {
    const page = window as unknown as { __e2eCipher?: unknown }
    page.__e2eCipher = undefined
    api.biometric.biometricCipher('e2e', options).then(
      (result) => {
        page.__e2eCipher = { ok: true, data: result.data }
      },
      (error: { code?: string; message?: string } | string) => {
        page.__e2eCipher =
          typeof error === 'string'
            ? { ok: false, code: null, message: error }
            : {
                ok: false,
                code: error.code ?? null,
                message: error.message ?? ''
              }
      }
    )
  }, options)
}

/** The outcome of the last {@link startCipher} call, or `null` while it is pending. */
async function cipherOutcome(): Promise<CipherOutcome | null> {
  return await tauri(
    () =>
      (window as unknown as { __e2eCipher?: CipherOutcome }).__e2eCipher ?? null
  )
}

/** Runs `biometricCipher`, answering its prompt with the enrolled finger. */
async function cipherWithFingerprint(
  options: CipherOptions
): Promise<CipherOutcome> {
  await startCipher(options)
  return await eventually(
    async () => {
      const outcome = await cipherOutcome()
      if (outcome) {
        return outcome
      }
      // Touching before the prompt is up is harmless, so keep at it.
      touchFingerprintSensor()
      throw new Error('biometricCipher did not settle')
    },
    { timeout: 60_000, interval: 1000 }
  )
}

async function encrypt(data: string): Promise<string> {
  const outcome = await cipherWithFingerprint({ dataToEncrypt: data })
  if (!outcome.ok) {
    throw new Error(`encryption failed: ${outcome.message} (${outcome.code})`)
  }
  return outcome.data
}

describePlugin('biometric', { mobileOnly: true }, () => {
  // Android only, where the lock screen state can be read over adb.
  const noLockCredential =
    platform === 'android' && androidHasNoLockCredential()
  const itNoLockCredential = noLockCredential ? it : it.skip

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

  const itAndroid = platform === 'android' ? it : it.skip

  itAndroid(
    'biometricCipher rejects without data to encrypt or decrypt',
    async () => {
      const message = await tauriError((api) =>
        api.biometric.biometricCipher('e2e', {})
      )
      expect(message).toMatch(/dataToEncrypt or dataToDecrypt/)
    }
  )

  // Enrolling changes the device's lock screen, so this only runs on an
  // emulator that has none (which it then restores).
  const describeEnrolled =
    noLockCredential && androidIsEmulator() ? describe : describe.skip

  describeEnrolled('biometricCipher with an enrolled fingerprint', () => {
    before(async () => {
      try {
        await enrollFingerprint()
      } finally {
        await returnToApp()
      }
      const status = await tauri((api) => api.biometric.checkStatus())
      expect(status.isAvailable).toBe(true)
    })

    after(() => {
      // Removing the PIN also removes the enrolled fingerprints.
      adb(['shell', 'locksettings', 'clear', '--old', PIN])
    })

    it('round-trips data through encryption and decryption', async () => {
      const plain = 'e2e sécret 🔐'
      const encrypted = await encrypt(plain)
      // Base64 `data;iv`
      expect(encrypted).toMatch(/^[A-Za-z0-9+/]+=*;[A-Za-z0-9+/]+=*$/)
      expect(encrypted).not.toContain(plain)

      const decrypted = await cipherWithFingerprint({
        dataToDecrypt: encrypted
      })
      expect(decrypted).toEqual({ ok: true, data: plain })
    })

    it('uses a fresh IV for every encryption', async () => {
      const first = await encrypt('same input')
      const second = await encrypt('same input')
      expect(first).not.toBe(second)
    })

    it('rejects decrypting data that was tampered with', async () => {
      const [data, iv] = (await encrypt('do not touch')).split(';')
      const bytes = Buffer.from(data, 'base64')
      bytes[0] ^= 0xff
      const outcome = await cipherWithFingerprint({
        dataToDecrypt: `${bytes.toString('base64')};${iv}`
      })
      expect(outcome.ok).toBe(false)
      expect(!outcome.ok && outcome.message).toMatch(/Failed to decrypt/)
    })

    it('rejects malformed encrypted data before prompting', async () => {
      await startCipher({ dataToDecrypt: 'not encrypted data' })
      const outcome = await eventually(
        async () => {
          const outcome = await cipherOutcome()
          if (!outcome) {
            throw new Error('biometricCipher did not settle')
          }
          return outcome
        },
        { timeout: 15_000 }
      )
      expect(outcome.ok).toBe(false)
      expect(!outcome.ok && outcome.message).toMatch(
        /Failed to initialize the cipher/
      )
    })

    it('rejects with userCancel when the prompt is canceled', async () => {
      await startCipher({ dataToEncrypt: 'canceled' })
      await browser.switchAppiumContext('NATIVE_APP')
      try {
        const cancel = browser.$(
          'android=new UiSelector().clickable(true).textMatches("(?i)^cancel$")'
        )
        await cancel.waitForDisplayed({ timeout: 30_000 })
        // A click while the prompt is still animating in is ignored, so keep
        // clicking until it is gone.
        await browser.waitUntil(
          async () => {
            if (!(await cancel.isExisting())) {
              return true
            }
            await cancel.click().catch(() => {})
            return false
          },
          {
            timeout: 15_000,
            interval: 500,
            timeoutMsg: 'the biometric prompt did not close'
          }
        )
      } finally {
        // The app comes back on its own once the prompt closes. Activating it
        // here instead would tear down a prompt that is still up, which the
        // plugin reports as systemCancel.
        await switchToWebview()
      }
      const outcome = await eventually(
        async () => {
          const outcome = await cipherOutcome()
          if (!outcome) {
            throw new Error('biometricCipher did not settle')
          }
          return outcome
        },
        { timeout: 15_000 }
      )
      expect(outcome).toMatchObject({ ok: false, code: 'userCancel' })
    })
  })
})
