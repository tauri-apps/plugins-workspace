// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

export async function getLastLink() {
  await window.__TAURI_INVOKE__<string | null>(
    "plugin:deep-link|get_last_link"
  );
}

// TODO: REGISTER EVENT LISTENER
