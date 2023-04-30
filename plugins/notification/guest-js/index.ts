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

/**
 * Options to send a notification.
 *
 * @since 1.0.0
 */
interface Options {
  /**
   * The notification identifier to reference this object later. Must be a 32-bit integer.
   */
  id?: number
  /**
   * Identifier of the {@link Channel} that deliveres this notification.
   * 
   * If the channel does not exist, the notification won't fire.
   * Make sure the channel exists with {@link listChannels} and {@link createChannel}.
   */
  channelId?: string
  /**
   * Notification title.
   */
  title: string
  /**
   * Optional notification body.
   * */
  body?: string
  /**
   * Schedule this notification to fire on a later time or a fixed interval.
   */
  schedule?: Schedule
  /**
   * Multiline text.
   * Changes the notification style to big text.
   * Cannot be used with `inboxLines`.
   */
  largeBody?: string
  /**
   * Detail text for the notification with `largeBody`, `inboxLines` or `groupSummary`.
   */
  summary?: string
  /**
   * Defines an action type for this notification.
   */
  actionTypeId?: string
  /**
   * Identifier used to group multiple notifications.
   * 
   * https://developer.apple.com/documentation/usernotifications/unmutablenotificationcontent/1649872-threadidentifier
   */
  group?: string
  /**
   * Instructs the system that this notification is the summary of a group on Android.
   */
  groupSummary?: boolean
  /**
   * The sound resource name. Only available on mobile.
   */
  sound?: string
  /**
   * List of lines to add to the notification.
   * Changes the notification style to inbox.
   * Cannot be used with `largeBody`.
   * 
   * Only supports up to 5 lines.
   */
  inboxLines?: string[]
  /**
   * Notification icon.
   * 
   * On Android the icon must be placed in the app's `res/drawable` folder.
   */
  icon?: string
  /**
   * Notification large icon (Android).
   * 
   * The icon must be placed in the app's `res/drawable` folder.
   */
  largeIcon?: string
  /**
   * Icon color on Android.
   */
  iconColor?: string
  /**
   * Notification attachments.
   */
  attachments?: Attachment[]
  /**
   * Extra payload to store in the notification.
   */
  extra?: { [key: string]: any }
  /**
   * If true, the notification cannot be dismissed by the user on Android.
   * 
   * An application service must manage the dismissal of the notification.
   * It is typically used to indicate a background task that is pending (e.g. a file download)
   * or the user is engaged with (e.g. playing music).
   */
  ongoing?: boolean
  /**
   * Automatically cancel the notification when the user clicks on it.
   */
  autoCancel?: boolean
}

type ScheduleInterval = {
  year?: number
  month?: number
  day?: number
  /**
   * 1 - Sunday
   * 2 - Monday
   * 3 - Tuesday
   * 4 - Wednesday
   * 5 - Thursday
   * 6 - Friday
   * 7 - Saturday
   */
  weekday?: number
  hour?: number
  minute?: number
  second?: number
}

enum ScheduleEvery {
  Year = 'Year',
  Month = 'Month',
  TwoWeeks = 'TwoWeeks',
  Week = 'Week',
  Day = 'Day',
  Hour = 'Hour',
  Minute = 'Minute',
  /**
   * Not supported on iOS.
   */
  Second = 'Second'
}

type ScheduleData = {
  kind: 'At',
  data: {
    date: Date
    repeating: boolean
  }
} | {
  kind: 'Interval',
  data: ScheduleInterval
} | {
  kind: 'Every',
  data: {
    interval: ScheduleEvery
  }
}

class Schedule {
  kind: string
  data: unknown

  private constructor(schedule: ScheduleData) {
    this.kind = schedule.kind
    this.data = schedule.data
  }

  static at(date: Date, repeating = false) {
    return new Schedule({ kind: 'At', data: { date, repeating } })
  }

  static interval(interval: ScheduleInterval) {
    return new Schedule({ kind: 'Interval', data: interval })
  }

  static every(kind: ScheduleEvery) {
    return new Schedule({ kind: 'Every', data: { interval: kind } })
  }
}

/**
 * Attachment of a notification.
 */
interface Attachment {
  /** Attachment identifier. */
  id: string
  /** Attachment URL. Accepts the `asset` and `file` protocols. */
  url: string
}

interface Action {
  id: string
  title: string
  requiresAuthentication?: boolean
  foreground?: boolean
  destructive?: boolean
  input?: boolean
  inputButtonTitle?: string
  inputPlaceholder?: string
}

interface ActionType {
  /**
   * The identifier of this action type
   */
  id: string
  /**
   * The list of associated actions
   */
  actions: Action[]
  hiddenPreviewsBodyPlaceholder?: string,
  customDismissAction?: boolean,
  allowInCarPlay?: boolean,
  hiddenPreviewsShowTitle?: boolean,
  hiddenPreviewsShowSubtitle?: boolean,
}

interface PendingNotification {
  id: number
  title?: string
  body?: string
  schedule: Schedule
}

interface ActiveNotification {
  id: number
  tag?: string
  title?: string
  body?: string
  group?: string
  groupSummary: boolean
  data: Record<string, string>
  extra: Record<string, unknown>
  attachments: Attachment[]
  actionTypeId?: string
  schedule?: Schedule
  sound?: string
}

enum Importance {
  None = 0,
  Min,
  Low,
  Default,
  High
}

enum Visibility {
  Secret = -1,
  Private,
  Public
}

interface Channel {
  id: string
  name: string
  description?: string
  sound?: string
  lights?: boolean
  lightColor?: string
  vibration?: boolean
  importance?: Importance
  visibility?: Visibility
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

/**
 * Register actions that are performed when the user clicks on the notification.
 * 
 * @example
 * ```typescript
 * import { registerActionTypes } from '@tauri-apps/api/notification';
 * await registerActionTypes([{
 *   id: 'tauri',
 *   actions: [{
 *     id: 'my-action',
 *     title: 'Settings'
 *   }]
 * }])
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function registerActionTypes(types: ActionType[]): Promise<void> {
  return invoke('plugin:notification|register_action_types', { types })
}

/**
 * Retrieves the list of pending notifications.
 * 
 * @example
 * ```typescript
 * import { pending } from '@tauri-apps/api/notification';
 * const pendingNotifications = await pending();
 * ```
 *
 * @returns A promise resolving to the list of pending notifications.
 *
 * @since 2.0.0
 */
async function pending(): Promise<PendingNotification[]> {
  return invoke('plugin:notification|get_pending')
}

/**
 * Cancels the pending notifications with the given list of identifiers.
 * 
 * @example
 * ```typescript
 * import { cancel } from '@tauri-apps/api/notification';
 * await cancel([-34234, 23432, 4311]);
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function cancel(notifications: number[]): Promise<void> {
  return invoke('plugin:notification|cancel', { notifications })
}

/**
 * Cancels all pending notifications.
 * 
 * @example
 * ```typescript
 * import { cancelAll } from '@tauri-apps/api/notification';
 * await cancelAll();
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function cancelAll(): Promise<void> {
  return invoke('plugin:notification|cancel')
}

/**
 * Retrieves the list of active notifications.
 * 
 * @example
 * ```typescript
 * import { active } from '@tauri-apps/api/notification';
 * const activeNotifications = await active();
 * ```
 *
 * @returns A promise resolving to the list of active notifications.
 *
 * @since 2.0.0
 */
async function active(): Promise<ActiveNotification[]> {
  return invoke('plugin:notification|get_active')
}

/**
 * Removes the active notifications with the given list of identifiers.
 * 
 * @example
 * ```typescript
 * import { cancel } from '@tauri-apps/api/notification';
 * await cancel([-34234, 23432, 4311])
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function removeActive(notifications: number[]): Promise<void> {
  return invoke('plugin:notification|remove_active', { notifications })
}

/**
 * Removes all active notifications.
 * 
 * @example
 * ```typescript
 * import { removeAllActive } from '@tauri-apps/api/notification';
 * await removeAllActive()
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function removeAllActive(): Promise<void> {
  return invoke('plugin:notification|remove_active')
}

/**
 * Removes all active notifications.
 * 
 * @example
 * ```typescript
 * import { createChannel, Importance, Visibility } from '@tauri-apps/api/notification';
 * await createChannel({
 *   id: 'new-messages',
 *   name: 'New Messages',
 *   lights: true,
 *   vibration: true,
 *   importance: Importance.Default,
 *   visibility: Visibility.Private
 * });
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function createChannel(channel: Channel): Promise<void> {
  return invoke('plugin:notification|create_channel', { ...channel })
}

/**
 * Removes the channel with the given identifier.
 * 
 * @example
 * ```typescript
 * import { removeChannel } from '@tauri-apps/api/notification';
 * await removeChannel();
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function removeChannel(id: string): Promise<void> {
  return invoke('plugin:notification|delete_channel', { id })
}

/**
 * Retrieves the list of notification channels.
 * 
 * @example
 * ```typescript
 * import { channels } from '@tauri-apps/api/notification';
 * const notificationChannels = await channels();
 * ```
 *
 * @returns A promise resolving to the list of notification channels.
 *
 * @since 2.0.0
 */
async function channels(): Promise<Channel[]> {
  return invoke('plugin:notification|getActive')
}

export type { Attachment, Options, Permission, Action, ActionType, PendingNotification, ActiveNotification, Channel }

export {
  Importance,
  Visibility,
  sendNotification,
  requestPermission,
  isPermissionGranted,
  registerActionTypes,
  pending,
  cancel,
  cancelAll,
  active,
  removeActive,
  removeAllActive,
  createChannel,
  removeChannel,
  channels
}
