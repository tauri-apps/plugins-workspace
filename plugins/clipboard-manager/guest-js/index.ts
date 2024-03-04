// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Read and write to the system clipboard.
 *
 * @module
 */

import { invoke } from "@tauri-apps/api/core";

type ClipResponse = Record<"plainText", { text: string }>;

/**
 * Writes plain text to the clipboard.
 * @example
 * ```typescript
 * import { writeText, readText } from '@tauri-apps/plugin-clipboard-manager';
 * await writeText('Tauri is awesome!');
 * assert(await readText(), 'Tauri is awesome!');
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function writeText(
  text: string,
  opts?: { label?: string },
): Promise<void> {
  return invoke("plugin:clipboard-manager|write", {
    data: {
      plainText: {
        label: opts?.label,
        text,
      },
    },
  });
}

/**
 * Gets the clipboard content as plain text.
 * @example
 * ```typescript
 * import { readText } from '@tauri-apps/plugin-clipboard-manager';
 * const clipboardText = await readText();
 * ```
 * @since 2.0.0
 */
async function readText(): Promise<string> {
  const kind: ClipResponse = await invoke("plugin:clipboard-manager|read");
  return kind.plainText.text;
}

/**
 * Writes HTML or fallbacks to write provided plain text to the clipboard.
 * @example
 * ```typescript
 * import { writeHtml, readHtml } from '@tauri-apps/plugin-clipboard-manager';
 * await writeHtml('<h1>Tauri is awesome!</h1>', 'plaintext');
 * await writeHtml('<h1>Tauri is awesome!</h1>', '<h1>Tauri is awesome</h1>'); // Will write "<h1>Tauri is awesome</h1>" as plain text
 * assert(await readText(), '<h1>Tauri is awesome!</h1>');
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function writeHtml(html: string, altHtml?: string): Promise<void> {
  return invoke("plugin:clipboard-manager|write_html", {
    data: {
      html: {
        html,
        altHtml,
      },
    },
  });
}

/**
 * Clears the clipboard.
 * @example
 * ```typescript
 * import { clear } from '@tauri-apps/plugin-clipboard-manager';
 * await clear();
 * ```
 * @since 2.0.0
 */
async function clear(): Promise<void> {
  await invoke("plugin:clipboard-manager|clear");
  return;
}

export { writeText, readText, writeHtml, clear };
