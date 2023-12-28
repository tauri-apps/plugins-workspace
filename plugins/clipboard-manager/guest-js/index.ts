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

type ClipImageResponse = Record<"image", { buffer: number[] }>;

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
  return invoke("plugin:clipboard|write_text", {
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
  const kind: ClipResponse = await invoke("plugin:clipboard|read_text");
  return kind.plainText.text;
}

/**
 * Gets the clipboard content as Uint8Array image.
 * @example
 * ```typescript
 * import { readImage } from '@tauri-apps/plugin-clipboard-manager';
 *
 * const clipboardImage = await readImage();
 * const blob = new Blob([clipboardImage.buffer], { type: 'image' })
 * const url = URL.createObjectURL(blob)
 * ```
 * @since 2.0.0
 */
async function readImage(): Promise<Uint8Array> {
  const kind: ClipImageResponse = await invoke("plugin:clipboard|read_image");
  return Uint8Array.from(kind.image.buffer);
}

/**
 * Writes image buffer to the clipboard.
 * @example
 * ```typescript
 * import { writeImage } from '@tauri-apps/plugin-clipboard-manager';
 * const buffer = [
 *   // A red pixel
 *   255, 0, 0, 255,
 *
 *  // A green pixel
 *   0, 255, 0, 255,
 * ];
 * await writeImage(buffer);
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function writeImage(buffer: Uint8Array | Array<number>): Promise<void> {
  return invoke("plugin:clipboard|write_image", {
    data: {
      image: {
        buffer: Array.isArray(buffer) ? buffer : Array.from(buffer),
      },
    },
  });
}

export { writeText, readText, readImage, writeImage };
