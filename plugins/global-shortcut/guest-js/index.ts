// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Register global shortcuts.
 *
 * @module
 */

import { invoke, Channel } from "@tauri-apps/api/core";

export type ShortcutHandler = (shortcut: string) => void;

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
 * await register('CommandOrControl+Shift+C', () => {
 *   console.log('Shortcut triggered');
 * });
 *
 * // or register multiple hotkeys at once
 * await register(['CommandOrControl+Shift+C', 'Alt+A'], (shortcut) => {
 *   console.log(`Shortcut ${shortcut} triggered`);
 * });
 * ```
 *
 * @param shortcut Shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 *
 * @since 2.0.0
 */
async function register(
  shortcuts: string | string[],
  handler: ShortcutHandler,
): Promise<void> {
  const h = new Channel<string>();
  h.onmessage = handler;

  return await invoke("plugin:global-shortcut|register", {
    shortcuts: Array.isArray(shortcuts) ? shortcuts : [shortcuts],
    handler: h,
  });
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
 * @param shortcut shortcut definition (modifiers and key separated by "+" e.g. CmdOrControl+Q), also accepts a list of shortcuts
 *
 * @since 2.0.0
 */
async function unregister(shortcuts: string | string[]): Promise<void> {
  return await invoke("plugin:global-shortcut|unregister", {
    shortcuts: Array.isArray(shortcuts) ? shortcuts : [shortcuts],
  });
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
 *
 * @since 2.0.0
 */
async function isRegistered(shortcut: string): Promise<boolean> {
  return await invoke("plugin:global-shortcut|is_registered", {
    shortcut,
  });
}

export { register, unregister, isRegistered };
