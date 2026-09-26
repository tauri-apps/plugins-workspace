// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Get and track the device's current position, mirroring the W3C Geolocation API.
 *
 * @module
 */

import {
  Channel,
  invoke,
  PermissionState,
  checkPermissions as checkPluginPermissions
} from '@tauri-apps/api/core'

/**
 * The GPS coordinates of a {@link Position}, along with the accuracy of each reading.
 */
export type Coordinates = {
  /**
   * Latitude in decimal degrees.
   */
  latitude: number
  /**
   * Longitude in decimal degrees.
   */
  longitude: number
  /**
   * Accuracy level of the latitude and longitude coordinates in meters.
   */
  accuracy: number
  /**
   * Accuracy level of the altitude coordinate in meters, if available.
   * Available on all iOS versions and on Android 8 and above.
   */
  altitudeAccuracy: number | null
  /**
   * The altitude the user is at, if available.
   */
  altitude: number | null
  /**
   * The speed the user is traveling, in meters per second, if available.
   */
  speed: number | null
  /**
   * The heading the user is facing, if available.
   */
  heading: number | null
}

/**
 * The current permission state for the geolocation APIs.
 */
export type PermissionStatus = {
  /**
   * Permission state for the location alias.
   *
   * On Android it requests/checks both ACCESS_COARSE_LOCATION and ACCESS_FINE_LOCATION permissions.
   *
   * On iOS it requests/checks location permissions.
   */
  location: PermissionState
  /**
   * Permissions state for the coarseLoaction alias.
   *
   * On Android it requests/checks ACCESS_COARSE_LOCATION.
   *
   * On Android 12+, users can choose between Approximate location (ACCESS_COARSE_LOCATION) and Precise location (ACCESS_FINE_LOCATION).
   *
   * On iOS it will have the same value as the `location` alias.
   */
  coarseLocation: PermissionState
}

/**
 * The individual permission aliases that can be requested with {@link requestPermissions}.
 *
 * `location` maps to both the coarse and fine location permissions on Android and to the standard
 * location permission on iOS. `coarseLocation` maps to the coarse location permission only on
 * Android, and behaves the same as `location` on iOS.
 */
export type PermissionType = 'location' | 'coarseLocation'

/**
 * A geolocation reading, as returned by {@link getCurrentPosition} and passed to the callback of {@link watchPosition}.
 */
export type Position = {
  /**
   * Creation time for these coordinates, in milliseconds since the Unix epoch.
   */
  timestamp: number
  /**
   * The GPS coordinates along with the accuracy of the data.
   */
  coords: Coordinates
}

/**
 * Options used to configure a {@link getCurrentPosition} or {@link watchPosition} request.
 */
export type PositionOptions = {
  /**
   * High accuracy mode (such as GPS, if available)
   * Will be ignored on Android 12+ if users didn't grant the ACCESS_FINE_LOCATION permission (`coarseLocation` permission).
   */
  enableHighAccuracy: boolean
  /**
   * The maximum wait time in milliseconds for location updates.
   * On Android the timeout gets ignored for getCurrentPosition.
   * Ignored on iOS
   */
  timeout: number
  /**
   * The maximum age in milliseconds of a possible cached position that is acceptable to return.
   * Default: 0
   * Ignored on iOS
   */
  maximumAge: number
}

/**
 * Registers a callback that is invoked with the device's position whenever it changes, similar to the W3C `navigator.geolocation.watchPosition` API. Pass the returned id to {@link clearWatch} to stop watching.
 *
 * @example
 * ```typescript
 * import { watchPosition } from '@tauri-apps/plugin-geolocation';
 * const watchId = await watchPosition(
 *   { enableHighAccuracy: true, timeout: 10000, maximumAge: 0 },
 *   (position, error) => {
 *     if (error) {
 *       console.error(error)
 *     } else {
 *       console.log(position)
 *     }
 *   }
 * );
 * ```
 *
 * @param options Configuration for the position watcher.
 * @param cb Callback invoked with the new {@link Position} on success, or `null` and an error message when a read fails.
 * @returns A promise resolving to the id of the registered watcher.
 * @since 2.0.0
 */
export async function watchPosition(
  options: PositionOptions & {requestUpdatesInBackground?: boolean},
  cb: (location: Position | null, error?: string) => void
): Promise<number> {
  const channel = new Channel<Position | string>()
  channel.onmessage = (message) => {
    if (typeof message === 'string') {
      cb(null, message)
    } else {
      cb(message)
    }
  }
  await invoke('plugin:geolocation|watch_position', {
    options,
    requestUpdatesInBackground: options.requestUpdatesInBackground ?? false,
    channel
  })
  return channel.id
}

/**
 * Returns the device's current position, similar to the W3C `navigator.geolocation.getCurrentPosition` API.
 *
 * @example
 * ```typescript
 * import { getCurrentPosition } from '@tauri-apps/plugin-geolocation';
 * const position = await getCurrentPosition();
 * ```
 *
 * @param options Configuration for the position request.
 * @returns A promise resolving to the current {@link Position}.
 * @since 2.0.0
 */
export async function getCurrentPosition(
  options?: PositionOptions
): Promise<Position> {
  return await invoke('plugin:geolocation|get_current_position', {
    options
  })
}

/**
 * Stops the position watcher registered with {@link watchPosition}.
 *
 * @example
 * ```typescript
 * import { clearWatch } from '@tauri-apps/plugin-geolocation';
 * await clearWatch(watchId);
 * ```
 *
 * @param channelId The id returned by {@link watchPosition}.
 * @since 2.0.0
 */
export async function clearWatch(channelId: number): Promise<void> {
  await invoke('plugin:geolocation|clear_watch', {
    channelId
  })
}

/**
 * Returns the current state of the geolocation permissions. Rejects if location services are disabled on the device.
 *
 * @example
 * ```typescript
 * import { checkPermissions } from '@tauri-apps/plugin-geolocation';
 * const permission = await checkPermissions();
 * ```
 *
 * @returns A promise resolving to the current {@link PermissionStatus}.
 * @since 2.0.0
 */
export async function checkPermissions(): Promise<PermissionStatus> {
  return await checkPluginPermissions('geolocation')
}

/**
 * Requests the given geolocation permissions, prompting the user if needed. Rejects if location services are disabled on the device.
 *
 * @example
 * ```typescript
 * import { requestPermissions } from '@tauri-apps/plugin-geolocation';
 * const permission = await requestPermissions(['location']);
 * ```
 *
 * @param permissions The permissions to request, or `null` to request all of them.
 * @returns A promise resolving to the resulting {@link PermissionStatus}.
 * @since 2.0.0
 */
export async function requestPermissions(
  permissions: PermissionType[] | null,
  requestUpdatesInBackground: boolean = false
): Promise<PermissionStatus> {
  return await invoke('plugin:geolocation|request_permissions', {
    permissions,
    requestUpdatesInBackground
  })
}
