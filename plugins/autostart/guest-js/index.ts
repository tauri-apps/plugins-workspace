// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/core";

export async function isEnabled(): Promise<boolean> {
  return await invoke("plugin:autostart|is_enabled");
}

export async function enable(): Promise<void> {
  await invoke("plugin:autostart|enable");
}

export async function disable(): Promise<void> {
  await invoke("plugin:autostart|disable");
}
