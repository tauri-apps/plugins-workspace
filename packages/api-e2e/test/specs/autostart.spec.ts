// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin } from '../helpers/index.js'

// Enabling autostart registers the app with the host for real (a Launch Agent
// on macOS, an XDG autostart entry on Linux, a `Run` registry value on
// Windows), so the spec puts back whatever state it found.

describePlugin('autostart', { desktopOnly: true }, () => {
  let initiallyEnabled = false

  before(async () => {
    initiallyEnabled = await tauri((api) => api.autostart.isEnabled())
  })

  after(async () => {
    await tauri(
      (api, enabled) =>
        enabled ? api.autostart.enable() : api.autostart.disable(),
      initiallyEnabled
    )
  })

  it('enable and disable toggle isEnabled', async () => {
    const states = await tauri(async (api) => {
      await api.autostart.enable()
      const enabled = await api.autostart.isEnabled()
      await api.autostart.disable()
      return { enabled, disabled: await api.autostart.isEnabled() }
    })
    expect(states).toEqual({ enabled: true, disabled: false })
  })

  it('enabling twice keeps it enabled', async () => {
    const enabled = await tauri(async (api) => {
      await api.autostart.enable()
      await api.autostart.enable()
      const enabled = await api.autostart.isEnabled()
      await api.autostart.disable()
      return enabled
    })
    expect(enabled).toBe(true)
  })

  it('disable resolves when autostart is already disabled', async () => {
    // Windows used to fail here, having no `Run` registry value to delete
    const disabled = await tauri(async (api) => {
      await api.autostart.enable()
      await api.autostart.disable()
      await api.autostart.disable()
      return !(await api.autostart.isEnabled())
    })
    expect(disabled).toBe(true)
  })
})
