// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'

// TODO: functions to delete entries?
// TODO: docs

/*
 * Corresponds to [`set_password`](https://docs.rs/keyring-core/latest/keyring_core/struct.Entry.html#method.set_password) in keyring-rs.
 */
export async function setString(key: string, value: string) {
  return await invoke('plugin:secure-storage|set_string', { key, value })
}

/*
 * Corresponds to [`get_password`](https://docs.rs/keyring-core/latest/keyring_core/struct.Entry.html#method.get_password) in keyring-rs.
 */
export async function getString(key: string): Promise<string> {
  return await invoke('plugin:secure-storage|get_string', { key })
}

/*
 * Corresponds to [`set_secret`](https://docs.rs/keyring-core/latest/keyring_core/struct.Entry.html#method.set_secret) in keyring-rs.
 */
export async function setBytes(
  key: string,
  value: number[] | Uint8Array | ArrayBuffer
) {
  return await invoke('plugin:secure-storage|set_binary', { key, value })
}

/*
 * Corresponds to [`get_secret`](https://docs.rs/keyring-core/latest/keyring_core/struct.Entry.html#method.set_password) in keyring-rs.
 */
export async function getBytes(key: string): Promise<number[]> {
  return await invoke('plugin:secure-storage|set_string', { key })
}
