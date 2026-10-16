// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Register global shortcuts.
 *
 * @module
 */

import { invoke, Channel } from '@tauri-apps/api/core'

/**
 * Payload sent to a shortcut handler when a registered shortcut is pressed or released.
 */
export interface ShortcutEvent {
  /** The shortcut definition that triggered this event, e.g. `CommandOrControl+Shift+C`. */
  shortcut: string
  /** Numeric identifier derived from the shortcut's modifiers and key. */
  id: number
  /** Whether the shortcut's key combination was pressed down or released. */
  state: 'Released' | 'Pressed'
}

/** Callback invoked with a {@link ShortcutEvent} whenever a registered shortcut changes state. */
export type ShortcutHandler = (event: ShortcutEvent) => void

/**
 * Register a global shortcut or a list of shortcuts.
 *
 * The handler is called when any of the registered shortcuts are pressed by the user.
 *
 * If the shortcut is already taken by another application, the handler will not be triggered.
 * Make sure the shortcut is as unique as possible while still taking user experience into consideration.
 *
 * @example
 * ```typescript
 * import { register } from '@tauri-apps/plugin-global-shortcut';
 *
 * // register a single hotkey
 * await register('CommandOrControl+Shift+C', (event) => {
 *   if (event.state === "Pressed") {
 *       console.log('Shortcut triggered');
 *   }
 * });
 *
 * // or register multiple hotkeys at once
 * await register(['CommandOrControl+Shift+C', 'Alt+A'], (event) => {
 *   console.log(`Shortcut ${event.shortcut} triggered`);
 * });
 * ```
 *
 * @param shortcuts A shortcut definition, or a list of shortcut definitions, with modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 *
 * @since 2.0.0
 */
async function register(
  shortcuts: string | string[],
  handler: ShortcutHandler
): Promise<void> {
  const h = new Channel<ShortcutEvent>()
  h.onmessage = handler

  return await invoke('plugin:global-shortcut|register', {
    shortcuts: Array.isArray(shortcuts) ? shortcuts : [shortcuts],
    handler: h
  })
}

/**
 * Unregister a global shortcut or a list of shortcuts.
 *
 * @example
 * ```typescript
 * import { unregister } from '@tauri-apps/plugin-global-shortcut';
 *
 * // unregister a single hotkey
 * await unregister('CmdOrControl+Space');
 *
 * // or unregister multiple hotkeys at the same time
 * await unregister(['CmdOrControl+Space', 'Alt+A']);
 * ```
 *
 * @param shortcuts A shortcut definition, or a list of shortcut definitions, with modifiers and key separated by "+" e.g. CmdOrControl+Q
 *
 * @since 2.0.0
 */
async function unregister(shortcuts: string | string[]): Promise<void> {
  return await invoke('plugin:global-shortcut|unregister', {
    shortcuts: Array.isArray(shortcuts) ? shortcuts : [shortcuts]
  })
}

/**
 * Unregister all global shortcuts.
 *
 * @example
 * ```typescript
 * import { unregisterAll } from '@tauri-apps/plugin-global-shortcut';
 * await unregisterAll();
 * ```
 * @since 2.0.0
 */
async function unregisterAll(): Promise<void> {
  return await invoke('plugin:global-shortcut|unregister_all', {})
}

/**
 * Determines whether the given shortcut is registered by this application or not.
 *
 * If the shortcut is registered by another application, it will still return `false`.
 *
 * @example
 * ```typescript
 * import { isRegistered } from '@tauri-apps/plugin-global-shortcut';
 * const isRegistered = await isRegistered('CommandOrControl+P');
 * ```
 *
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @returns A promise resolving to whether the shortcut is currently registered by this application.
 *
 * @since 2.0.0
 */
async function isRegistered(shortcut: string): Promise<boolean> {
  return await invoke('plugin:global-shortcut|is_registered', {
    shortcut
  })
}

export { register, unregister, unregisterAll, isRegistered }
