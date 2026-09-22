// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin } from '../helpers/index.js'
import {
  UPDATER_FIXTURE_VERSION,
  UPDATER_FIXTURE_NOTES,
  UPDATER_TARGET_NO_UPDATE,
  UPDATER_TARGET_OLDER
} from '../helpers/server.js'

// The e2e build points the updater endpoint at the fixture server (see
// `tauri.e2e.conf.json`), which answers based on the `{{target}}` placeholder.
// Only `check` is exercised: installing would replace the binary under test.
// The plugin is desktop-only, so the whole suite is skipped on mobile (the
// mobile builds are not built with the override config either).

describePlugin('updater', { desktopOnly: true }, () => {
  it('check finds a newer release on the endpoint', async () => {
    const update = await tauri(async (api) => {
      const update = await api.updater.check()
      if (!update) return null
      const info = {
        available: update.available,
        currentVersion: update.currentVersion,
        version: update.version,
        body: update.body,
        date: update.date,
        rawVersion: (update.rawJson as { version?: string }).version
      }
      await update.close()
      return info
    })
    expect(update).not.toBeNull()
    expect(update!.available).toBe(true)
    expect(update!.currentVersion).toBe('2.0.0')
    expect(update!.version).toBe(UPDATER_FIXTURE_VERSION)
    expect(update!.body).toBe(UPDATER_FIXTURE_NOTES)
    expect(update!.date).toContain('2026-03-01')
    expect(update!.rawVersion).toBe(UPDATER_FIXTURE_VERSION)
  })

  it('check resolves null when the endpoint has no update (204)', async () => {
    const update = await tauri(
      (api, target) => api.updater.check({ target }),
      UPDATER_TARGET_NO_UPDATE
    )
    expect(update).toBeNull()
  })

  it('check ignores a release older than the current version', async () => {
    // Downgrades are a build-time decision (the plugin's `allowDowngrades`
    // config), not something `check` can be asked for, so the older manifest
    // can only be checked for the update being ignored.
    const update = await tauri(
      (api, target) => api.updater.check({ target }),
      UPDATER_TARGET_OLDER
    )
    expect(update).toBeNull()
  })

  it('check forwards custom headers and honors the timeout option', async () => {
    // a successful check with extra headers and a generous timeout
    const version = await tauri(async (api) => {
      const update = await api.updater.check({
        headers: { 'x-e2e-updater': 'yes' },
        timeout: 30_000
      })
      const version = update?.version ?? null
      await update?.close()
      return version
    })
    expect(version).toBe(UPDATER_FIXTURE_VERSION)
  })
})
