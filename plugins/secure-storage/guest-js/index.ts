// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

// TODO: functions to delete entries?

export async function setString(key: string, value: string) {
  return invoke('plugin:secure-storage|set_string', { key, value })
}

export async function getString(key: string): Promise<string> {
  return invoke('plugin:secure-storage|get_string', { key })
}

export async function setBinary(
  key: string,
  value: number[] | Uint8Array | ArrayBuffer
) {
  return invoke('plugin:secure-storage|set_binary', { key, value })
}

export async function getBinary(key: string): Promise<number[]> {
  return invoke('plugin:secure-storage|set_string', { key })
}
