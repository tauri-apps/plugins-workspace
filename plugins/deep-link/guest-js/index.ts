// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Set your Tauri application as the default handler for a URL, or check which URL(s) it was opened with.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'
import { type UnlistenFn, listen } from '@tauri-apps/api/event'

/**
 * Get the current URLs that triggered the deep link. Use this on app load to check whether your app was started via a deep link.
 *
 * @example
 * ```typescript
 * import { getCurrent } from '@tauri-apps/plugin-deep-link';
 * const urls = await getCurrent();
 * ```
 *
 * #### Platform-specific
 *
 * - **Windows / Linux:** This function reads the command line arguments and checks if there's only one value, which must be an URL with scheme matching one of the configured values.
 *   Note that you must manually check the arguments when registering deep link schemes dynamically with {@link register}.
 *   Additionally, the deep link might have been provided as a CLI argument so you should check if its format matches what you expect.
 *
 * @returns A promise resolving to the list of URLs that triggered the deep link, or `null` if the app was not started via a deep link.
 * @since 2.0.0
 */
export async function getCurrent(): Promise<string[] | null> {
  return await invoke('plugin:deep-link|get_current')
}

/**
 * Register the app as the default handler for the specified protocol.
 *
 * @param protocol The name of the protocol without `://`. For example, if you want your app to handle `tauri://` links, call this method with `tauri` as the protocol.
 *
 * @example
 * ```typescript
 * import { register } from '@tauri-apps/plugin-deep-link';
 * await register("my-scheme");
 * ```
 *
 * #### Platform-specific
 *
 * - **macOS / Android / iOS:** Unsupported.
 *
 * @returns A promise that resolves once the protocol has been registered.
 * @since 2.0.0
 */
export async function register(protocol: string): Promise<null> {
  return await invoke('plugin:deep-link|register', { protocol })
}

/**
 * Unregister the app as the default handler for the specified protocol.
 *
 * @param protocol The name of the protocol without `://`.
 *
 * @example
 * ```typescript
 * import { unregister } from '@tauri-apps/plugin-deep-link';
 * await unregister("my-scheme");
 * ```
 *
 * #### Platform-specific
 *
 * - **Windows:** Requires admin rights if the protocol is registered on the local machine (this can happen when registered from the NSIS installer when the install mode is set to both or per machine).
 * - **Linux / FreeBSD:** Can only unregister the scheme if it was initially registered with {@link register}. May not work on older distros.
 * - **macOS / Android / iOS:** Unsupported.
 *
 * @returns A promise that resolves once the protocol has been unregistered.
 * @since 2.0.0
 */
export async function unregister(protocol: string): Promise<null> {
  return await invoke('plugin:deep-link|unregister', { protocol })
}

/**
 * Check whether the app is the default handler for the specified protocol.
 *
 * @param protocol The name of the protocol without `://`.
 *
 * @example
 * ```typescript
 * import { isRegistered } from '@tauri-apps/plugin-deep-link';
 * await isRegistered("my-scheme");
 * ```
 *
 * #### Platform-specific
 *
 * - **macOS / Android / iOS:** Unsupported.
 *
 * @returns A promise resolving to `true` if the app is the default handler for the protocol, `false` otherwise.
 * @since 2.0.0
 */
export async function isRegistered(protocol: string): Promise<boolean> {
  return await invoke('plugin:deep-link|is_registered', { protocol })
}

/**
 * Helper function for the `deep-link://new-url` event to run a function each time the protocol is triggered while the app is running. Use {@link getCurrent} on app load to check whether your app was started via a deep link.
 *
 * @param handler The function to call with the list of URLs the app was requested to open, every time this happens while the app is running.
 *
 * @example
 * ```typescript
 * import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
 * await onOpenUrl((urls) => { console.log(urls) });
 * ```
 *
 * #### Platform-specific
 *
 * - **Windows / Linux:** Unsupported without the single-instance plugin. The OS will spawn a new app instance passing the URL as a CLI argument.
 *
 * @returns A promise resolving to a function that unregisters the event listener.
 * @since 2.0.0
 */
export async function onOpenUrl(
  handler: (urls: string[]) => void
): Promise<UnlistenFn> {
  return await listen<string[]>('deep-link://new-url', (event) => {
    handler(event.payload)
  })
}
