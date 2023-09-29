// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Read and write to the system clipboard.
 *
 * @module
 */

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

interface Clip<K, T> {
  kind: K;
  options: T;
}

type ClipResponse = Clip<"PlainText", string>;

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
  return window.__TAURI_INVOKE__("plugin:clipboard|write", {
    data: {
      kind: "PlainText",
      options: {
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
  const kind: ClipResponse = await window.__TAURI_INVOKE__(
    "plugin:clipboard|read",
  );
  return kind.options;
}

export { writeText, readText };
