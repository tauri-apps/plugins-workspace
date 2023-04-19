// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Send toast notifications (brief auto-expiring OS window element) to your user.
 * Can also be used with the Notification Web API.
 *
 * This package is also accessible with `window.__TAURI__.notification` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.notification`](https://tauri.app/v1/api/config/#allowlistconfig.notification) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "notification": {
 *         "all": true // enable all notification APIs
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */

import { invoke } from '@tauri-apps/api/tauri'

type PermissionState = 'granted' | 'denied'

  ; (function () {
    let permissionSettable = false
    let permissionValue = 'default'

    function isPermissionGranted() {
      if (window.Notification.permission !== 'default') {
        return Promise.resolve(window.Notification.permission === 'granted')
      }
      return invoke('plugin:notification|is_permission_granted')
    }

    function setNotificationPermission(value: 'default' | PermissionState) {
      permissionSettable = true
      // @ts-expect-error we can actually set this value on the webview
      window.Notification.permission = value
      permissionSettable = false
    }

    function requestPermission() {
      return invoke<PermissionState>('plugin:notification|request_permission')
        .then(function (permission) {
          setNotificationPermission(permission)
          return permission
        })
    }

    function sendNotification(options: Options) {
      if (typeof options === 'object') {
        Object.freeze(options)
      }

      return invoke('plugin:notification|notify', {
        options: typeof options === 'string'
          ? {
            title: options
          }
          : options
      })
    }

    // @ts-expect-error unfortunately we can't implement the whole type, so we overwrite it with our own version
    window.Notification = function (title, options) {
      const opts = options || {}
      sendNotification(
        Object.assign(opts, {
          title: title
        })
      )
    }

    window.Notification.requestPermission = requestPermission

    Object.defineProperty(window.Notification, 'permission', {
      enumerable: true,
      get: function () {
        return permissionValue
      },
      set: function (v) {
        if (!permissionSettable) {
          throw new Error('Readonly property')
        }
        permissionValue = v
      }
    })

    isPermissionGranted().then(function (response) {
      if (response === null) {
        setNotificationPermission('default')
      } else {
        setNotificationPermission(response ? 'granted' : 'denied')
      }
    })
  })()

/**
 * Options to send a notification.
 *
 * @since 1.0.0
 */
interface Options {
  /** Notification title. */
  title: string
  /** Optional notification body. */
  body?: string
  /** Optional notification icon. */
  icon?: string
}

/** Possible permission values. */
type Permission = 'granted' | 'denied' | 'default'

/**
 * Checks if the permission to send notifications is granted.
 * @example
 * ```typescript
 * import { isPermissionGranted } from '@tauri-apps/api/notification';
 * const permissionGranted = await isPermissionGranted();
 * ```
 *
 * @since 1.0.0
 */
async function isPermissionGranted(): Promise<boolean> {
  if (window.Notification.permission !== 'default') {
    return Promise.resolve(window.Notification.permission === 'granted')
  }
  return invoke('plugin:notification|is_permission_granted')
}

/**
 * Requests the permission to send notifications.
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission } from '@tauri-apps/api/notification';
 * let permissionGranted = await isPermissionGranted();
 * if (!permissionGranted) {
 *   const permission = await requestPermission();
 *   permissionGranted = permission === 'granted';
 * }
 * ```
 *
 * @returns A promise resolving to whether the user granted the permission or not.
 *
 * @since 1.0.0
 */
async function requestPermission(): Promise<Permission> {
  return window.Notification.requestPermission()
}

/**
 * Sends a notification to the user.
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/api/notification';
 * let permissionGranted = await isPermissionGranted();
 * if (!permissionGranted) {
 *   const permission = await requestPermission();
 *   permissionGranted = permission === 'granted';
 * }
 * if (permissionGranted) {
 *   sendNotification('Tauri is awesome!');
 *   sendNotification({ title: 'TAURI', body: 'Tauri is awesome!' });
 * }
 * ```
 *
 * @since 1.0.0
 */
function sendNotification(options: Options | string): void {
  if (typeof options === 'string') {
    // eslint-disable-next-line no-new
    new window.Notification(options)
  } else {
    // eslint-disable-next-line no-new
    new window.Notification(options.title, options)
  }
}

export type { Options, Permission }

export { sendNotification, requestPermission, isPermissionGranted }
