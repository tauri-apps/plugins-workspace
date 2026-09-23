// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin } from '../helpers/index.js'

// Enabling autostart registers the app with the host for real (a Launch Agent
// on macOS, an XDG autostart entry on Linux, a `Run` registry value on
// Windows), so the spec puts back whatever state it found.
//
// `disable` is only called while autostart is enabled: on Windows it fails
// when there is no `Run` value to delete.

describePlugin('autostart', { desktopOnly: true }, () => {
  let initiallyEnabled = false

  before(async () => {
    initiallyEnabled = await tauri((api) => api.autostart.isEnabled())
  })

  after(async () => {
    await tauri(async (api, enabled) => {
      if ((await api.autostart.isEnabled()) === enabled) return
      await (enabled ? api.autostart.enable() : api.autostart.disable())
    }, initiallyEnabled)
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
    const states = await tauri(async (api) => {
      await api.autostart.enable()
      await api.autostart.enable()
      const enabled = await api.autostart.isEnabled()
      await api.autostart.disable()
      return { enabled, disabled: await api.autostart.isEnabled() }
    })
    expect(states).toEqual({ enabled: true, disabled: false })
  })
})
