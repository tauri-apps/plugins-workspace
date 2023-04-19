// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Read and write to the system clipboard.
 *
 * The APIs must be added to [`tauri.allowlist.clipboard`](https://tauri.app/v1/api/config/#allowlistconfig.clipboard) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "clipboard": {
 *         "all": true, // enable all Clipboard APIs
 *         "writeText": true,
 *         "readText": true
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/tauri'

interface Clip<K, T> {
  kind: K
  options: T
}

type ClipResponse = Clip<'PlainText', string>

/**
 * Writes plain text to the clipboard.
 * @example
 * ```typescript
 * import { writeText, readText } from 'tauri-plugin-clipboard-api';
 * await writeText('Tauri is awesome!');
 * assert(await readText(), 'Tauri is awesome!');
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0.
 */
async function writeText(text: string, opts?: { label?: string }): Promise<void> {
  return invoke('plugin:clipboard|write', {
    data: {
      kind: 'PlainText',
      options: {
        label: opts?.label,
        text
      }
    }
  })
}

/**
 * Gets the clipboard content as plain text.
 * @example
 * ```typescript
 * import { readText } from 'tauri-plugin-clipboard-api';
 * const clipboardText = await readText();
 * ```
 * @since 1.0.0.
 */
async function readText(): Promise<string> {
  const kind: ClipResponse = await invoke('plugin:clipboard|read')
  return kind.options
}

export { writeText, readText }
