// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import os from 'node:os'
import { expect } from '@wdio/globals'
import { tauri, describePlugin } from '../helpers/index.js'

const nodePlatformToTauri: Record<string, string> = {
  linux: 'linux',
  win32: 'windows',
  darwin: 'macos',
  freebsd: 'freebsd',
  openbsd: 'openbsd'
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

describePlugin('os', () => {
  it('platform, type and family match the host', async () => {
    const info = await tauri((api) => ({
      platform: api.os.platform(),
      type: api.os.type(),
      family: api.os.family()
    }))
    expect(info.platform).toBe(nodePlatformToTauri[process.platform])
    expect(info.type).toBe(nodePlatformToTauri[process.platform])
    expect(info.family).toBe(process.platform === 'win32' ? 'windows' : 'unix')
  })

  it('arch matches the host', async () => {
    expect(await tauri((api) => api.os.arch())).toBe(
      nodeArchToTauri[process.arch]
    )
  })

  it('eol and exeExtension match the host conventions', async () => {
    const info = await tauri((api) => ({
      eol: api.os.eol(),
      exeExtension: api.os.exeExtension()
    }))
    expect(info.eol).toBe(os.EOL)
    expect(info.exeExtension).toBe(process.platform === 'win32' ? 'exe' : '')
  })

  it('version reports a non-empty OS version', async () => {
    const version = await tauri((api) => api.os.version())
    expect(version.length).toBeGreaterThan(0)
  })

  it('locale is null or a language tag', async () => {
    const locale = await tauri((api) => api.os.locale())
    if (locale !== null) {
      // e.g. `en-US`
      const [language, ...subtags] = locale.split(/[-_]/)
      expect(language).toMatch(/^[A-Za-z]{2,3}$/)
      for (const subtag of subtags) {
        expect(subtag).toMatch(/^[A-Za-z0-9]+$/)
      }
    }
  })

  it('hostname matches the host', async () => {
    const hostname = await tauri((api) => api.os.hostname())
    expect(hostname).not.toBeNull()
    // Windows can report the name in a different case than Node does.
    expect(hostname!.toLowerCase()).toBe(os.hostname().toLowerCase())
  })
})
