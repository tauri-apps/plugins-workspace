// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import type { invoke } from "@tauri-apps/api/primitives";

/** @ignore */
declare global {
  interface Window {
    __TAURI_INTERNALS__: {
      invoke: typeof invoke;
    };
  }
}

export async function isEnabled(): Promise<boolean> {
  return await window.__TAURI_INTERNALS__.invoke("plugin:autostart|is_enabled");
}

export async function enable(): Promise<void> {
  await window.__TAURI_INTERNALS__.invoke("plugin:autostart|enable");
}

export async function disable(): Promise<void> {
  await window.__TAURI_INTERNALS__.invoke("plugin:autostart|disable");
}
