// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Send toast notifications (brief auto-expiring OS window element) to your user.
 * Can also be used with the Notification Web API.
 *
 * @module
 */

import {
  invoke,
  type PluginListener,
  addPluginListener
} from '@tauri-apps/api/core'

export type { PermissionState } from '@tauri-apps/api/core'

/**
 * Options to send a notification.
 *
 * @since 2.0.0
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
   * The sound resource name or file path for the notification.
   *
   * Platform specific behavior:
   * - On macOS: use system sounds (e.g., "Ping", "Blow") or sound files in the app bundle
   * - On Linux: use XDG theme sounds (e.g., "message-new-instant") or file paths
   * - On Windows: use file paths to sound files (.wav format)
   * - On Mobile: use resource names
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
  extra?: Record<string, unknown>
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
  /**
   * Changes the notification presentation to be silent on iOS (no badge, no sound, not listed).
   */
  silent?: boolean
  /**
   * Notification visibility.
   */
  visibility?: Visibility
  /**
   * Sets the number of items this notification represents on Android.
   */
  number?: number
}

interface ScheduleInterval {
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
  Year = 'year',
  Month = 'month',
  TwoWeeks = 'twoWeeks',
  Week = 'week',
  Day = 'day',
  Hour = 'hour',
  Minute = 'minute',
  /**
   * Not supported on iOS.
   */
  Second = 'second'
}

class Schedule {
  at:
    | {
        date: Date
        repeating: boolean
        allowWhileIdle: boolean
      }
    | undefined

  interval:
    | {
        interval: ScheduleInterval
        allowWhileIdle: boolean
      }
    | undefined

  every:
    | {
        interval: ScheduleEvery
        count: number
        allowWhileIdle: boolean
      }
    | undefined

  static at(date: Date, repeating = false, allowWhileIdle = false): Schedule {
    return {
      at: { date, repeating, allowWhileIdle },
      interval: undefined,
      every: undefined
    }
  }

  static interval(
    interval: ScheduleInterval,
    allowWhileIdle = false
  ): Schedule {
    return {
      at: undefined,
      interval: { interval, allowWhileIdle },
      every: undefined
    }
  }

  static every(
    kind: ScheduleEvery,
    count: number,
    allowWhileIdle = false
  ): Schedule {
    return {
      at: undefined,
      interval: undefined,
      every: { interval: kind, count, allowWhileIdle }
    }
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
  hiddenPreviewsBodyPlaceholder?: string
  customDismissAction?: boolean
  allowInCarPlay?: boolean
  hiddenPreviewsShowTitle?: boolean
  hiddenPreviewsShowSubtitle?: boolean
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

interface ActionPerformedNotification {
  actionId: string
  id?: number
  inputValue?: string
  notification?: ActiveNotification | null
}

type RawPendingRecord = Record<string, unknown> & {
  nameValuePairs?: unknown
  value?: unknown
  length?: number
}

type PendingActionsRaw =
  | ActionPerformedNotification
  | ActionPerformedNotification[]
  | RawPendingRecord
  | RawPendingRecord[]
  | null
  | undefined

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

/**
 * Checks if the permission to send notifications is granted.
 * @example
 * ```typescript
 * import { isPermissionGranted } from '@tauri-apps/plugin-notification';
 * const permissionGranted = await isPermissionGranted();
 * ```
 *
 * @since 2.0.0
 */
async function isPermissionGranted(): Promise<boolean> {
  if (window.Notification.permission !== 'default') {
    return await Promise.resolve(window.Notification.permission === 'granted')
  }
  return await invoke('plugin:notification|is_permission_granted')
}

/**
 * Requests the permission to send notifications.
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission } from '@tauri-apps/plugin-notification';
 * let permissionGranted = await isPermissionGranted();
 * if (!permissionGranted) {
 *   const permission = await requestPermission();
 *   permissionGranted = permission === 'granted';
 * }
 * ```
 *
 * @returns A promise resolving to whether the user granted the permission or not.
 *
 * @since 2.0.0
 */
async function requestPermission(): Promise<NotificationPermission> {
  return await window.Notification.requestPermission()
}

/**
 * Sends a notification to the user.
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/plugin-notification';
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
 * @since 2.0.0
 */
function sendNotification(options: Options | string): void {
  if (typeof options === 'string') {
    new window.Notification(options)
  } else {
    new window.Notification(options.title, options)
  }
}

/**
 * Register actions that are performed when the user clicks on the notification.
 *
 * @example
 * ```typescript
 * import { registerActionTypes } from '@tauri-apps/plugin-notification';
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
  await invoke('plugin:notification|register_action_types', { types })
}

/**
 * Retrieves the list of pending notifications.
 *
 * @example
 * ```typescript
 * import { pending } from '@tauri-apps/plugin-notification';
 * const pendingNotifications = await pending();
 * ```
 *
 * @returns A promise resolving to the list of pending notifications.
 *
 * @since 2.0.0
 */
async function pending(): Promise<PendingNotification[]> {
  return await invoke('plugin:notification|get_pending')
}

/**
 * Cancels the pending notifications with the given list of identifiers.
 *
 * @example
 * ```typescript
 * import { cancel } from '@tauri-apps/plugin-notification';
 * await cancel([-34234, 23432, 4311]);
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function cancel(notifications: number[]): Promise<void> {
  await invoke('plugin:notification|cancel', { notifications })
}

/**
 * Cancels all pending notifications.
 *
 * @example
 * ```typescript
 * import { cancelAll } from '@tauri-apps/plugin-notification';
 * await cancelAll();
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function cancelAll(): Promise<void> {
  await invoke('plugin:notification|cancel')
}

/**
 * Retrieves the list of active notifications.
 *
 * @example
 * ```typescript
 * import { active } from '@tauri-apps/plugin-notification';
 * const activeNotifications = await active();
 * ```
 *
 * @returns A promise resolving to the list of active notifications.
 *
 * @since 2.0.0
 */
async function active(): Promise<ActiveNotification[]> {
  return await invoke('plugin:notification|get_active')
}

/**
 * Removes the active notifications with the given list of identifiers.
 *
 * @example
 * ```typescript
 * import { cancel } from '@tauri-apps/plugin-notification';
 * await cancel([-34234, 23432, 4311])
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function removeActive(
  notifications: Array<{ id: number; tag?: string }>
): Promise<void> {
  await invoke('plugin:notification|remove_active', { notifications })
}

/**
 * Removes all active notifications.
 *
 * @example
 * ```typescript
 * import { removeAllActive } from '@tauri-apps/plugin-notification';
 * await removeAllActive()
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function removeAllActive(): Promise<void> {
  await invoke('plugin:notification|remove_active')
}

/**
 * Creates a notification channel.
 *
 * @example
 * ```typescript
 * import { createChannel, Importance, Visibility } from '@tauri-apps/plugin-notification';
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
  await invoke('plugin:notification|create_channel', { ...channel })
}

/**
 * Removes the channel with the given identifier.
 *
 * @example
 * ```typescript
 * import { removeChannel } from '@tauri-apps/plugin-notification';
 * await removeChannel();
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function removeChannel(id: string): Promise<void> {
  await invoke('plugin:notification|delete_channel', { id })
}

/**
 * Retrieves the list of notification channels.
 *
 * @example
 * ```typescript
 * import { channels } from '@tauri-apps/plugin-notification';
 * const notificationChannels = await channels();
 * ```
 *
 * @returns A promise resolving to the list of notification channels.
 *
 * @since 2.0.0
 */
async function channels(): Promise<Channel[]> {
  return await invoke('plugin:notification|listChannels')
}

async function onNotificationReceived(
  cb: (notification: Options) => void
): Promise<PluginListener> {
  return await addPluginListener('notification', 'notification', cb)
}

function normalisePendingActions(
  pending: PendingActionsRaw
): ActionPerformedNotification[] {
  const normalisedActions: ActionPerformedNotification[] = []
  const seenObjects = new WeakSet<object>()
  const seenActionKeys = new Set<string>()

  const isRawPendingRecord = (value: unknown): value is RawPendingRecord => {
    return !!value && typeof value === 'object' && !Array.isArray(value)
  }

  const toRecord = (value: unknown): RawPendingRecord | null => {
    if (!isRawPendingRecord(value)) {
      return null
    }

    const record = value
    const wrapped = record.nameValuePairs
    if (isRawPendingRecord(wrapped)) {
      return toRecord(wrapped)
    }

    return record
  }

  const buildAction = (
    candidate: unknown
  ): ActionPerformedNotification | null => {
    const record = toRecord(candidate)
    if (!record) {
      return null
    }

    const actionId = record.actionId
    if (typeof actionId !== 'string' || actionId.length === 0) {
      return null
    }

    const action: ActionPerformedNotification = {
      actionId
    }

    const rawId = record.id
    if (typeof rawId === 'number') {
      action.id = rawId
    } else if (typeof rawId === 'string') {
      const parsedId = Number.parseInt(rawId, 10)
      if (!Number.isNaN(parsedId)) {
        action.id = parsedId
      }
    }

    if (typeof record.inputValue === 'string') {
      action.inputValue = record.inputValue
    }

    const toNumber = (value: unknown): number | null => {
      if (typeof value === 'number' && Number.isFinite(value)) {
        return value
      }
      if (typeof value === 'string') {
        const parsed = Number.parseInt(value, 10)
        if (!Number.isNaN(parsed)) {
          return parsed
        }
      }
      return null
    }

    const toStringRecord = (value: unknown): Record<string, string> => {
      if (!isRawPendingRecord(value)) {
        return {}
      }

      const output: Record<string, string> = {}
      for (const [key, item] of Object.entries(value)) {
        if (typeof item === 'string') {
          // Dynamic key passthrough is intentional here: this function only normalises
          // already-received bridge payload objects into a plain string record.
          // eslint-disable-next-line security/detect-object-injection
          output[key] = item
        }
      }
      return output
    }

    const toUnknownRecord = (value: unknown): Record<string, unknown> => {
      if (!isRawPendingRecord(value)) {
        return {}
      }
      return value
    }

    const coerceActiveNotification = (
      value: unknown
    ): ActiveNotification | null => {
      const notificationRecord = toRecord(value)
      if (!notificationRecord) {
        return null
      }

      const id = toNumber(notificationRecord.id)
      if (id === null) {
        return null
      }

      const activeNotification: ActiveNotification = {
        id,
        groupSummary:
          typeof notificationRecord.groupSummary === 'boolean'
            ? notificationRecord.groupSummary
            : false,
        data: toStringRecord(notificationRecord.data),
        extra: toUnknownRecord(notificationRecord.extra),
        attachments: Array.isArray(notificationRecord.attachments)
          ? (notificationRecord.attachments as Attachment[])
          : []
      }

      if (typeof notificationRecord.tag === 'string') {
        activeNotification.tag = notificationRecord.tag
      }
      if (typeof notificationRecord.title === 'string') {
        activeNotification.title = notificationRecord.title
      }
      if (typeof notificationRecord.body === 'string') {
        activeNotification.body = notificationRecord.body
      }
      if (typeof notificationRecord.group === 'string') {
        activeNotification.group = notificationRecord.group
      }
      if (typeof notificationRecord.actionTypeId === 'string') {
        activeNotification.actionTypeId = notificationRecord.actionTypeId
      }
      if (typeof notificationRecord.sound === 'string') {
        activeNotification.sound = notificationRecord.sound
      }
      if (
        notificationRecord.schedule &&
        isRawPendingRecord(notificationRecord.schedule)
      ) {
        activeNotification.schedule =
          notificationRecord.schedule as unknown as Schedule
      }

      return activeNotification
    }

    if ('notification' in record) {
      action.notification = coerceActiveNotification(record.notification)
    }

    return action
  }

  const addAction = (action: ActionPerformedNotification): void => {
    const key = `${action.id ?? ''}|${action.actionId}|${action.inputValue ?? ''}`
    if (seenActionKeys.has(key)) {
      return
    }
    seenActionKeys.add(key)
    normalisedActions.push(action)
  }

  const walk = (value: unknown): void => {
    if (!value || typeof value !== 'object') {
      return
    }

    if (Array.isArray(value)) {
      for (const entry of value) {
        walk(entry)
      }
      return
    }

    const objectValue = value
    if (seenObjects.has(objectValue)) {
      return
    }
    seenObjects.add(objectValue)

    if (!isRawPendingRecord(value)) {
      return
    }
    const record = value

    const directAction = buildAction(record)
    if (directAction) {
      addAction(directAction)
      return
    }

    const wrappedValue = record.value
    if (wrappedValue !== undefined) {
      walk(wrappedValue)
    }

    // Some host bridges return array-like objects (`{ 0: ..., length: N }`).
    if (typeof record.length === 'number') {
      for (let index = 0; index < record.length; index += 1) {
        walk(record[String(index)])
      }
    }

    for (const entry of Object.values(record)) {
      walk(entry)
    }
  }

  walk(pending)

  return normalisedActions
}

/**
 * Registers a listener for notification action events.
 *
 * @since 2.0.0
 */
function onAction(
  cb: (notification: ActionPerformedNotification) => void
): Promise<PluginListener>
/**
 * Registers a listener for notification action events.
 *
 * @deprecated Use the `ActionPerformedNotification` callback type.
 * @since 2.0.0
 */
function onAction(cb: (notification: Options) => void): Promise<PluginListener>
async function onAction(
  cb:
    | ((notification: ActionPerformedNotification) => void)
    | ((notification: Options) => void)
): Promise<PluginListener> {
  const actionCallback = cb as (notification: ActionPerformedNotification) => void
  const listener = await addPluginListener(
    'notification',
    'actionPerformed',
    (notification: ActionPerformedNotification) => actionCallback(notification)
  )
  try {
    const pendingResult = await invoke<PendingActionsRaw>(
      'plugin:notification|register_action_listener_ready'
    )
    const pending = normalisePendingActions(pendingResult)
    console.debug(
      `[NotificationPlugin] register_action_listener_ready replay count=${pending.length}`
    )
    for (const notification of pending) {
      actionCallback(notification)
    }
  } catch {
    // Older plugin versions and non-Android targets may not implement this command.
  }
  return listener
}

export type {
  Attachment,
  Options,
  Action,
  ActionType,
  PendingNotification,
  ActiveNotification,
  ActionPerformedNotification,
  Channel,
  ScheduleInterval
}

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
  channels,
  onNotificationReceived,
  onAction,
  Schedule,
  ScheduleEvery
}
