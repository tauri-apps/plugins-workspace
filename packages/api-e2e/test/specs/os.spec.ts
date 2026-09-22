// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import os from 'node:os'
import { expect } from '@wdio/globals'
import {
  tauri,
  describePlugin,
  itDesktop,
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

  it('version reports a non-empty OS version', async () => {
    const version = await tauri((api) => api.os.version())
    expect(version.length).toBeGreaterThan(0)
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

  it('hostname reports a name', async () => {
    const hostname = await tauri((api) => api.os.hostname())
    expect(hostname).not.toBeNull()
    expect(hostname!.length).toBeGreaterThan(0)
  })

  itDesktop('hostname matches the host', async () => {
    const hostname = await tauri((api) => api.os.hostname())
    // Windows can report the name in a different case than Node does.
    expect(hostname!.toLowerCase()).toBe(os.hostname().toLowerCase())
  })
})
