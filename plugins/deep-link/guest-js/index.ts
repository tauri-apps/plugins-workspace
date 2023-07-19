// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

export async function getLastLink(): Promise<string[] | null> {
  return await window.__TAURI_INVOKE__("plugin:deep-link|get_last_link");
}

// TODO: onUrlEvent function (helper function for the event listener)
