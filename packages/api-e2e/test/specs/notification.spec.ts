// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import { tauri, describePlugin, itDesktop } from '../helpers/index.js'

// Whether a notification actually shows up depends on the desktop session
// (a notification daemon on Linux, the app's registration on Windows and
// macOS), which the suite cannot observe. The specs cover the permission
// model and that sending does not error out synchronously.
//
// The permission specs are desktop-only: mobile starts out ungranted and
// `requestPermission` puts up a system dialog the session would then block on.

describePlugin('notification', () => {
  itDesktop('permission is granted on desktop', async () => {
    const result = await tauri(async (api) => ({
      granted: await api.notification.isPermissionGranted(),
      requested: await api.notification.requestPermission()
    }))
    expect(result).toEqual({ granted: true, requested: 'granted' })
  })

  itDesktop('the plugin overrides window.Notification', async () => {
    const result = await tauri(async () => ({
      permission: window.Notification.permission,
      requested: await window.Notification.requestPermission()
    }))
    expect(result).toEqual({ permission: 'granted', requested: 'granted' })
  })

  it('sendNotification accepts a title string and an options object', async () => {
    const result = await tauri((api) => {
      api.notification.sendNotification('notification from e2e')
      api.notification.sendNotification({
        title: 'notification from e2e',
        body: 'with a body',
        sound: 'default'
      })
      return true
    })
    expect(result).toBe(true)
  })

  it('new Notification() goes through the plugin', async () => {
    const result = await tauri(() => {
      const notification = new window.Notification(
        'window.Notification from e2e',
        {
          body: 'created through the DOM API'
        }
      )
      return typeof notification === 'object'
    })
    expect(result).toBe(true)
  })
})
