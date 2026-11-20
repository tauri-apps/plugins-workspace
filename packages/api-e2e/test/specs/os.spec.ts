// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import os from 'node:os'
import { expect } from '@wdio/globals'
import {
  tauri,
  describePlugin,
  itDesktop,
  itOn,
  platform,
  isMobile
} from '../helpers/index.js'

// Everything the plugin reports is baked in at compile time, so it describes
// the *app's* platform — the host for the desktop suite, but the emulator or
// simulator for the mobile ones, which is why nothing here compares against
// Node's view of the host unless the two are the same machine.

const nodePlatformToTauri: Record<string, string> = {
  linux: 'linux',
  win32: 'windows',
  darwin: 'macos',
  freebsd: 'freebsd',
  openbsd: 'openbsd',
  android: 'android',
  ios: 'ios'
}

const nodeArchToTauri: Record<string, string> = {
  x64: 'x86_64',
  ia32: 'x86',
  arm64: 'aarch64',
  arm: 'arm',
  riscv64: 'riscv64',
  ppc64: 'powerpc64',
  s390x: 's390x'
}

// `Arch` in the plugin's guest-js.
const architectures = [
  'x86',
  'x86_64',
  'arm',
  'aarch64',
  'mips',
  'mips64',
  'powerpc',
  'powerpc64',
  'riscv64',
  's390x',
  'sparc64'
]

describePlugin('os', () => {
  it('platform, type and family match the target', async () => {
    const info = await tauri((api) => ({
      platform: api.os.platform(),
      type: api.os.type(),
      family: api.os.family()
    }))
    // eslint-disable-next-line security/detect-object-injection
    expect(info.platform).toBe(nodePlatformToTauri[platform])
    // eslint-disable-next-line security/detect-object-injection
    expect(info.type).toBe(nodePlatformToTauri[platform])
    expect(info.family).toBe(platform === 'win32' ? 'windows' : 'unix')
  })

  it('arch reports a known architecture', async () => {
    const arch = await tauri((api) => api.os.arch())
    if (isMobile) {
      // The device/simulator is not necessarily the host's architecture (see
      // `E2E_ANDROID_TARGET` / `E2E_IOS_TARGET`).
      expect(architectures).toContain(arch)
    } else {
      expect(arch).toBe(nodeArchToTauri[process.arch])
    }
  })

  it('eol and exeExtension match the platform conventions', async () => {
    const info = await tauri((api) => ({
      eol: api.os.eol(),
      exeExtension: api.os.exeExtension()
    }))
    expect(info.eol).toBe(platform === 'win32' ? '\r\n' : '\n')
    expect(info.exeExtension).toBe(platform === 'win32' ? 'exe' : '')
  })

  it('locale is null or a language tag', async () => {
    const locale = await tauri((api) => api.os.locale())
    // The plugin forwards the environment's POSIX locale as-is, so a host with
    // no locale configured (CI runners default to `LANG=C.UTF-8`) reports the
    // POSIX default instead of a language tag.
    if (locale !== null && locale !== 'C' && locale !== 'POSIX') {
      // e.g. `en-US`
      const [language, ...subtags] = locale.split(/[-_]/)
      expect(language).toMatch(/^[A-Za-z]{2,3}$/)
      for (const subtag of subtags) {
        expect(subtag).toMatch(/^[A-Za-z0-9]+$/)
      }
    }
  })

  itDesktop('hostname matches the host', async () => {
    const hostname = await tauri((api) => api.os.hostname())
    // Windows can report the name in a different case than Node does.
    expect(hostname!.toLowerCase()).toBe(os.hostname().toLowerCase())
  })

  itOn(['android', 'ios'], 'version is known on mobile', async () => {
    const version = await tauri((api) => api.os.version())
    // os_info reports `Unknown` on mobile, the plugin reads it itself there
    expect(version).toMatch(/^\d[\d.]*$/)
  })

  it('deviceInfo reports the device', async () => {
    const info = await tauri((api) => api.os.deviceInfo())
    // every platform reports the memory size
    expect(typeof info.totalMemory).toBe('number')
    expect(info.totalMemory!).toBeGreaterThan(0)
    if (isMobile) {
      expect(info.model).toMatch(/\S/)
      expect(info.manufacturer).toMatch(/\S/)
      // usually an emulator/simulator, but E2E_ANDROID_DEVICE can be a real one
      expect(typeof info.isVirtual).toBe('boolean')
      expect(info.webviewVersion).toMatch(/^\d[\d.]*$/)
    } else {
      expect(info.isVirtual).toBeNull()
      expect(info.webviewVersion).toBeNull()
    }
    if (platform === 'ios' || platform === 'darwin') {
      expect(info.manufacturer).toBe('Apple')
    }
  })

  itOn('ios', 'deviceInfo reports the simulator', async () => {
    const info = await tauri((api) => api.os.deviceInfo())
    // the machine identifier of the simulated device, not the host's `arm64`
    expect(info.model).toMatch(/^(iPhone|iPad|iPod)\d+,\d+$/)
    expect(info.isVirtual).toBe(true)
  })

  it('deviceName is a string on mobile and null on desktop', async () => {
    const name = await tauri((api) => api.os.deviceName())
    if (platform === 'ios') {
      expect(name).toMatch(/\S/)
    } else if (platform === 'android') {
      // Android only reports it when the device name setting is set
      if (name !== null) {
        expect(name).toMatch(/\S/)
      }
    } else {
      expect(name).toBeNull()
    }
  })

  it('batteryInfo is an object on mobile and null on desktop', async () => {
    const battery = await tauri((api) => api.os.batteryInfo())
    if (!isMobile) {
      expect(battery).toBeNull()
      return
    }
    expect(battery).not.toBeNull()
    // the iOS simulator reports no battery
    if (battery!.level !== null) {
      expect(battery!.level).toBeGreaterThanOrEqual(0)
      expect(battery!.level).toBeLessThanOrEqual(1)
    }
    if (battery!.isCharging !== null) {
      expect(typeof battery!.isCharging).toBe('boolean')
    }
  })
})
