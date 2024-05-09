// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/core";
import { type UnlistenFn, listen } from "@tauri-apps/api/event";

export async function getCurrent(): Promise<string[] | null> {
  return await invoke<string[] | null>("plugin:deep-link|get_current");

  // return await invoke("plugin:deep-link|get_current");
}

export async function onOpenUrl(
  handler: (urls: string[]) => void,
): Promise<UnlistenFn> {
  const current = await getCurrent();
  if (current != null) {
    handler(current);
  }

  return await listen<string[]>("deep-link://new-url", (event) => {
    handler(event.payload);
  });
}
