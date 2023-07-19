// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { transformCallback } from "@tauri-apps/api/tauri";

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

export async function getCurrent(): Promise<string[] | null> {
  // TODO: replace with `invoke` on next alpha
  return await window
      .__TAURI_INVOKE__<string[] | null>("plugin:deep-link|get_current")
    
  // return await invoke("plugin:deep-link|get_current");
}

export async function onOpenUrl(handler: (urls: string[]) => void): Promise<void> {
  const current = await getCurrent()
  if (current != null) {
    handler(current)
  }

  // TODO: Replace with `listen` on next alpha
  return await window.__TAURI_INVOKE__("plugin:event|listen", {
    event: "deep-link://new-url",
    windowLabel: "main",
    handler: transformCallback((event) => handler(event.payload)),
  });
  //return await listen<string[]>("deep-link://new-url", (event) => handler(event.payload))
}
