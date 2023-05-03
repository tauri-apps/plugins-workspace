// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Register global shortcuts.
 *
 * The APIs must be added to [`tauri.allowlist.globalShortcut`](https://tauri.app/v1/api/config/#allowlistconfig.globalshortcut) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "globalShortcut": {
 *         "all": true // enable all global shortcut APIs
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */

import { invoke, transformCallback } from "@tauri-apps/api/tauri";

export type ShortcutHandler = (shortcut: string) => void;

/**
 * Register a global shortcut.
 * @example
 * ```typescript
 * import { register } from 'tauri-plugin-global-shortcut-api';
 * await register('CommandOrControl+Shift+C', () => {
 *   console.log('Shortcut triggered');
 * });
 * ```
 *
 * @param shortcut Shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 *
 * @since 1.0.0
 */
async function register(
  shortcut: string,
  handler: ShortcutHandler
): Promise<void> {
  return await invoke("plugin:globalShortcut|register", {
    shortcut,
    handler: transformCallback(handler),
  });
}

/**
 * Register a collection of global shortcuts.
 * @example
 * ```typescript
 * import { registerAll } from 'tauri-plugin-global-shortcut-api';
 * await registerAll(['CommandOrControl+Shift+C', 'Ctrl+Alt+F12'], (shortcut) => {
 *   console.log(`Shortcut ${shortcut} triggered`);
 * });
 * ```
 *
 * @param shortcuts Array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 *
 * @since 1.0.0
 */
async function registerAll(
  shortcuts: string[],
  handler: ShortcutHandler
): Promise<void> {
  return await invoke("plugin:globalShortcut|register_all", {
    shortcuts,
    handler: transformCallback(handler),
  });
}

/**
 * Determines whether the given shortcut is registered by this application or not.
 *
 * If the shortcut is registered by another application, it will still return `false`.
 *
 * @example
 * ```typescript
 * import { isRegistered } from 'tauri-plugin-global-shortcut-api';
 * const isRegistered = await isRegistered('CommandOrControl+P');
 * ```
 *
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 *
 * @since 1.0.0
 */
async function isRegistered(shortcut: string): Promise<boolean> {
  return await invoke("plugin:globalShortcut|is_registered", {
    shortcut,
  });
}

/**
 * Unregister a global shortcut.
 * @example
 * ```typescript
 * import { unregister } from 'tauri-plugin-global-shortcut-api';
 * await unregister('CmdOrControl+Space');
 * ```
 *
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 *
 * @since 1.0.0
 */
async function unregister(shortcut: string): Promise<void> {
  return await invoke("plugin:globalShortcut|unregister", {
    shortcut,
  });
}

/**
 * Unregisters all shortcuts registered by the application.
 * @example
 * ```typescript
 * import { unregisterAll } from 'tauri-plugin-global-shortcut-api';
 * await unregisterAll();
 * ```
 *
 * @since 1.0.0
 */
async function unregisterAll(): Promise<void> {
  return await invoke("plugin:globalShortcut|unregister_all");
}

export { register, registerAll, isRegistered, unregister, unregisterAll };
