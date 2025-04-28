// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
 * #### - **Windows / Linux**: This function reads the command line arguments and checks if there's only one value, which must be an URL with scheme matching one of the configured values.
 * Note that you must manually check the arguments when registering deep link schemes dynamically with [`Self::register`].
 * Additionally, the deep link might have been provided as a CLI argument so you should check if its format matches what you expect..
 *
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
 * #### - **macOS / Android / iOS**: Unsupported.
 *
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
 * #### - **macOS / Linux / Android / iOS**: Unsupported.
 *
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
 * #### - **macOS / Android / iOS**: Unsupported.
 *
 * @since 2.0.0
 */
export async function isRegistered(protocol: string): Promise<boolean> {
  return await invoke('plugin:deep-link|is_registered', { protocol })
}

/**
 * Helper function for the `deep-link://new-url` event to run a function each time the protocol is triggered while the app is running. Use `getCurrent` on app load to check whether your app was started via a deep link.
 *
 * @param protocol The name of the protocol without `://`.
 *
 * @example
 * ```typescript
 * import { onOpenUrl } from '@tauri-apps/plugin-deep-link';
 * await onOpenUrl((urls) => { console.log(urls) });
 * ```
 *
 * #### - **Windows / Linux**: Unsupported without the single-instance plugin. The OS will spawn a new app instance passing the URL as a CLI argument.
 *
 * @since 2.0.0
 */
export async function onOpenUrl(
  handler: (urls: string[]) => void
): Promise<UnlistenFn> {
  return await listen<string[]>('deep-link://new-url', (event) => {
    handler(event.payload)
  })
}
