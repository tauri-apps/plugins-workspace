// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  itDesktop,
  itOn
} from '../helpers/index.js'

// Whether a notification actually shows up depends on the desktop session
// (a notification daemon on Linux, the app's registration on Windows and
// macOS), which the suite cannot observe, and sending is fire-and-forget, so
// only the permission model is covered on desktop.
//
// The permission specs are desktop-only: mobile starts out ungranted and
// `requestPermission` puts up a system dialog the session would then block on.
//
// Scheduling, action types, channels, the pending/active lists and the
// listeners are only implemented on mobile. None of them need the permission,
// so they are covered there on an app that has not been granted it.

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

  itOn(
    ['android', 'ios'],
    'action types register, and cancel/remove leave nothing pending or active',
    async () => {
      const result = await tauri(async (api) => {
        await api.notification.registerActionTypes([
          {
            id: 'e2e-actions',
            actions: [
              { id: 'reply', title: 'Reply', input: true },
              { id: 'dismiss', title: 'Dismiss', destructive: true }
            ]
          }
        ])
        await api.notification.cancel([424242])
        await api.notification.cancelAll()
        await api.notification.removeActive([{ id: 424242 }])
        await api.notification.removeAllActive()
        return {
          pending: await api.notification.pending(),
          active: await api.notification.active()
        }
      })
      expect(result).toEqual({ pending: [], active: [] })
    }
  )

  itOn('android', 'channels can be created, listed and removed', async () => {
    const result = await tauri(async (api) => {
      const { Importance, Visibility } = api.notification
      await api.notification.createChannel({
        id: 'e2e-channel',
        name: 'e2e channel',
        description: 'created by the e2e suite',
        importance: Importance.High,
        visibility: Visibility.Public,
        vibration: true
      })
      const created = (await api.notification.channels()).find(
        (channel) => channel.id === 'e2e-channel'
      )
      await api.notification.removeChannel('e2e-channel')
      const removed = (await api.notification.channels()).some(
        (channel) => channel.id === 'e2e-channel'
      )
      return {
        created: created && {
          name: created.name,
          description: created.description,
          importance: created.importance,
          vibration: created.vibration
        },
        removed
      }
    })
    expect(result).toEqual({
      created: {
        name: 'e2e channel',
        description: 'created by the e2e suite',
        importance: 4, // Importance.High
        vibration: true
      },
      removed: false
    })
  })

  itOn('ios', 'channels are not implemented', async () => {
    for (const call of ['create', 'remove', 'list'] as const) {
      const error = await tauriError(
        (api, call) =>
          call === 'create'
            ? api.notification.createChannel({ id: 'e2e', name: 'e2e' })
            : call === 'remove'
              ? api.notification.removeChannel('e2e')
              : api.notification.channels(),
        call
      )
      expect(error).toMatch(/not implemented/)
    }
  })

  itOn(
    ['android', 'ios'],
    'onNotificationReceived and onAction listeners register and unregister',
    async () => {
      const result = await tauri(async (api) => {
        const received = await api.notification.onNotificationReceived(() => {})
        const action = await api.notification.onAction(() => {})
        await received.unregister()
        await action.unregister()
        return { received: received.event, action: action.event }
      })
      expect(result).toEqual({
        received: 'notification',
        action: 'actionPerformed'
      })
    }
  )
})
