// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Read and write to the system clipboard.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'
import { Image, transformImage } from '@tauri-apps/api/image'

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
  opts?: { label?: string }
): Promise<void> {
  await invoke('plugin:clipboard-manager|write_text', {
    label: opts?.label,
    text
  })
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
  return await invoke('plugin:clipboard-manager|read_text')
}

/**
 * Writes image buffer to the clipboard.
 *
 * #### Platform-specific
 *
 * - **Android / iOS:** Not supported.
 *
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
async function writeImage(
  image: string | Image | Uint8Array | ArrayBuffer | number[]
): Promise<void> {
  await invoke('plugin:clipboard-manager|write_image', {
    image: transformImage(image)
  })
}

/**
 * Gets the clipboard content as Uint8Array image.
 *
 * #### Platform-specific
 *
 * - **Android / iOS:** Not supported.
 *
 * @example
 * ```typescript
 * import { readImage } from '@tauri-apps/plugin-clipboard-manager';
 *
 * const clipboardImage = await readImage();
 * const blob = new Blob([await clipboardImage.rbga()], { type: 'image' })
 * const url = URL.createObjectURL(blob)
 * ```
 * @since 2.0.0
 */
async function readImage(): Promise<Image> {
  return await invoke<number>('plugin:clipboard-manager|read_image').then(
    (rid) => new Image(rid)
  )
}

/**
 * * Writes HTML or fallbacks to write provided plain text to the clipboard.
 *
 * #### Platform-specific
 *
 * - **Android / iOS:** Not supported.
 *
 * @example
 * ```typescript
 * import { writeHtml } from '@tauri-apps/plugin-clipboard-manager';
 * await writeHtml('<h1>Tauri is awesome!</h1>', 'plaintext');
 * // The following will write "<h1>Tauri is awesome</h1>" as plain text
 * await writeHtml('<h1>Tauri is awesome!</h1>', '<h1>Tauri is awesome</h1>');
 * // we can read html data only as a string so there's just readText(), no readHtml()
 * assert(await readText(), '<h1>Tauri is awesome!</h1>');
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 2.0.0
 */
async function writeHtml(html: string, altText?: string): Promise<void> {
  await invoke('plugin:clipboard-manager|write_html', {
    html,
    altText
  })
}

/**
 * Clears the clipboard.
 *
 * #### Platform-specific
 *
 * - **Android:** Only supported on SDK 28+. For older releases we write an empty string to the clipboard instead.
 *
 * @example
 * ```typescript
 * import { clear } from '@tauri-apps/plugin-clipboard-manager';
 * await clear();
 * ```
 * @since 2.0.0
 */
async function clear(): Promise<void> {
  await invoke('plugin:clipboard-manager|clear')
}

export { writeText, readText, writeHtml, clear, readImage, writeImage }
