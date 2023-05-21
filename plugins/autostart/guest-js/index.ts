// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

export async function isEnabled(): Promise<boolean> {
  return await window.__TAURI_INVOKE__("plugin:autostart|is_enabled");
}

export async function enable(): Promise<void> {
  await window.__TAURI_INVOKE__("plugin:autostart|enable");
}

export async function disable(): Promise<void> {
  await window.__TAURI_INVOKE__("plugin:autostart|disable");
}
