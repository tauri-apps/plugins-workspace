// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Control the device flashlight (torch) on Android and iOS.
 *
 * On Android the torch keeps working while the barcode scanner plugin holds the camera.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * Turns the torch on.
 *
 * Rejects if the device has no torch, the torch is temporarily unavailable (e.g. the device is
 * overheating) or the camera is held by another app.
 *
 * @example
 * ```typescript
 * import { enable } from '@tauri-apps/plugin-torch'
 * await enable()
 * ```
 */
export async function enable(): Promise<void> {
  await invoke('plugin:torch|enable')
}

/**
 * Turns the torch off.
 *
 * Rejects in the same cases as {@link enable}.
 *
 * @example
 * ```typescript
 * import { disable } from '@tauri-apps/plugin-torch'
 * await disable()
 * ```
 */
export async function disable(): Promise<void> {
  await invoke('plugin:torch|disable')
}

/**
 * Flips the torch.
 *
 * Rejects in the same cases as {@link enable}.
 *
 * @example
 * ```typescript
 * import { toggle } from '@tauri-apps/plugin-torch'
 * const on = await toggle()
 * ```
 *
 * @returns Whether the torch is now on.
 */
export async function toggle(): Promise<boolean> {
  return await invoke<{ enabled: boolean }>('plugin:torch|toggle').then(
    (r) => r.enabled
  )
}

/**
 * Whether the device has a torch.
 *
 * @example
 * ```typescript
 * import { isAvailable } from '@tauri-apps/plugin-torch'
 * if (await isAvailable()) {
 *   // show the torch button
 * }
 * ```
 */
export async function isAvailable(): Promise<boolean> {
  return await invoke<{ available: boolean }>('plugin:torch|is_available').then(
    (r) => r.available
  )
}

/**
 * Whether the torch is currently on.
 *
 * This reads the hardware state, so it also reflects the torch being turned off by the system,
 * another app, or a barcode scan ending.
 *
 * On iOS, the torch turned on from Control Center or by another app is not visible to the app,
 * so this returns `false` for it.
 *
 * @example
 * ```typescript
 * import { isEnabled } from '@tauri-apps/plugin-torch'
 * const on = await isEnabled()
 * ```
 */
export async function isEnabled(): Promise<boolean> {
  return await invoke<{ enabled: boolean }>('plugin:torch|is_enabled').then(
    (r) => r.enabled
  )
}
