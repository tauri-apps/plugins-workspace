// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Injected script that replaces `window.alert` and `window.confirm` with implementations backed
 * by native dialogs.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

window.alert = function (message: string) {
  void invoke('plugin:dialog|message', {
    message: message.toString()
  })
}

// @ts-expect-error tauri does not have sync IPC :(
window.confirm = async function (message: string) {
  return await invoke('plugin:dialog|confirm', {
    message: message.toString()
  })
}
