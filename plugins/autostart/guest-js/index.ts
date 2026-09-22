// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Automatically launch your application at startup.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * Checks whether autostart is enabled for the application.
 * @example
 * ```typescript
 * import { isEnabled } from '@tauri-apps/plugin-autostart';
 * const enabled = await isEnabled();
 * ```
 *
 * @returns A promise resolving to `true` if the application launches at startup, `false` otherwise.
 * @since 2.0.0
 */
export async function isEnabled(): Promise<boolean> {
  return await invoke('plugin:autostart|is_enabled')
}

/**
 * Enables autostart for the application.
 * @example
 * ```typescript
 * import { enable } from '@tauri-apps/plugin-autostart';
 * await enable();
 * ```
 *
 * @since 2.0.0
 */
export async function enable(): Promise<void> {
  await invoke('plugin:autostart|enable')
}

/**
 * Disables autostart for the application.
 * @example
 * ```typescript
 * import { disable } from '@tauri-apps/plugin-autostart';
 * await disable();
 * ```
 *
 * @since 2.0.0
 */
export async function disable(): Promise<void> {
  await invoke('plugin:autostart|disable')
}
