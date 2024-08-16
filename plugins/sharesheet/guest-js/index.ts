// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/core";

export interface SharesheetOptions {
  // Android only
  mimeType?: string;
  thumbnailUri?: string;

  // Android & iOS
  title?: string;
}

/**
 * Opens the Sharesheet to share the specified text.
 *
 * ```javascript
 * import { shareText } from "@tauri-apps/plugin-sharesheet";
 * await shareText('I am a shared message');
 * ```
 * @param text
 * @param options
 * @returns
 */
export async function shareText(
  text: string,
  options?: SharesheetOptions,
): Promise<void> {
  await invoke("plugin:sharesheet|share_text", {
    text,
    ...options,
  });
}
