// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import path from 'node:path'
import { spawnSync } from 'node:child_process'
import { expect } from '@wdio/globals'
import {
  tauri,
  tauriError,
  describePlugin,
  eventually,
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
//
// On Android, what the system actually posted (`when` and the chronometer
// flags) is read back from `dumpsys notification` over adb; Appium's
// `autoGrantPermissions` has granted POST_NOTIFICATIONS so they are posted.

const appId = 'com.tauri.api'

/** `adb` from the Android SDK (else `PATH`), on `E2E_ANDROID_DEVICE` if set. */
function adb(args: string[]): string {
  const sdk = process.env.ANDROID_HOME ?? process.env.ANDROID_SDK_ROOT
  const binary = sdk ? path.join(sdk, 'platform-tools', 'adb') : 'adb'
  const device = process.env.E2E_ANDROID_DEVICE
    ? ['-s', process.env.E2E_ANDROID_DEVICE]
    : []
  const result = spawnSync(binary, [...device, ...args], {
    encoding: 'utf8',
    timeout: 20_000
  })
  if (result.status !== 0) {
    throw new Error(`adb ${args.join(' ')} failed: ${result.stderr}`)
  }
  return result.stdout
}

interface PostedNotification {
  when: number
  showChronometer: boolean
  chronometerCountDown: boolean
}

/**
 * The notification with `id` the app has posted, as the system's notification
 * service records it. Throws while it is not posted (for {@link eventually}).
 */
function postedNotification(id: number): PostedNotification {
  const dump = adb(['shell', 'dumpsys', 'notification', '--noredact'])
  const record = dump
    .split(/(?=^\s*NotificationRecord\()/m)
    // the record's key is `<user>|<package>|<id>|<tag>|<uid>`
    .find((block) => block.includes(`|${appId}|${id}|`))
  if (!record) {
    throw new Error(`notification ${id} is not posted`)
  }
  const when = /\bwhen=(\d+)/.exec(record)
  if (!when) {
    throw new Error(
      `no \`when\` in the record of notification ${id}:\n${record}`
    )
  }
  const flag = (key: string) => record.includes(`android.${key}=Boolean (true)`)
  return {
    when: Number(when[1]),
    showChronometer: flag('showChronometer'),
    chronometerCountDown: flag('chronometerCountDown')
  }
}

/** Cancels pending and removes posted notifications. */
async function clearNotifications(): Promise<void> {
  await tauri(async (api) => {
    await api.notification.cancelAll()
    await api.notification.removeAllActive()
    return null
  })
}

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

  itOn('android', '`when` and `usesChronometer` are posted', async () => {
    const when = Date.now() - 60_000
    try {
      await tauri((api, when) => {
        api.notification.sendNotification({
          id: 7101,
          title: 'e2e chronometer',
          when,
          usesChronometer: true
        })
        return null
      }, when)
      expect(await eventually(() => postedNotification(7101))).toEqual({
        when,
        showChronometer: true,
        chronometerCountDown: false
      })
    } finally {
      await clearNotifications()
    }
  })

  itOn('android', '`chronometerCountDown` is posted', async () => {
    const when = Date.now() + 10 * 60_000
    try {
      await tauri((api, when) => {
        api.notification.sendNotification({
          id: 7102,
          title: 'e2e countdown',
          when,
          usesChronometer: true,
          chronometerCountDown: true
        })
        return null
      }, when)
      expect(await eventually(() => postedNotification(7102))).toEqual({
        when,
        showChronometer: true,
        chronometerCountDown: true
      })
    } finally {
      await clearNotifications()
    }
  })

  // Scheduled notifications are built when scheduled and posted by an alarm,
  // which used to overwrite `when` with the time it fired. Without the exact
  // alarm permission the alarm is inexact, hence the generous timeout.
  itOn('android', 'a scheduled notification keeps its `when`', async () => {
    const when = Date.now() + 10 * 60_000
    try {
      await tauri(
        (api, when, at) => {
          api.notification.sendNotification({
            id: 7103,
            title: 'e2e scheduled countdown',
            schedule: api.notification.Schedule.at(new Date(at), false, true),
            when,
            usesChronometer: true,
            chronometerCountDown: true
          })
          return null
        },
        when,
        Date.now() + 3000
      )
      expect(
        await eventually(() => postedNotification(7103), { timeout: 90_000 })
      ).toEqual({ when, showChronometer: true, chronometerCountDown: true })
    } finally {
      await clearNotifications()
    }
  })

  itOn(
    'android',
    'a scheduled notification without `when` shows when it fired',
    async () => {
      const at = Date.now() + 3000
      try {
        await tauri((api, at) => {
          api.notification.sendNotification({
            id: 7104,
            title: 'e2e scheduled',
            schedule: api.notification.Schedule.at(new Date(at), false, true)
          })
          return null
        }, at)
        const posted = await eventually(() => postedNotification(7104), {
          timeout: 90_000
        })
        // The emulator's clock follows the host's, give or take a little.
        expect(posted.when).toBeGreaterThan(at - 2000)
        expect(posted.when).toBeLessThan(Date.now() + 2000)
      } finally {
        await clearNotifications()
      }
    }
  )

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
