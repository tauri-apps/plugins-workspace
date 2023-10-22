// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/primitives";

window.alert = function (message: string) {
  invoke("plugin:dialog|message", {
    message: message.toString(),
  });
};

// @ts-expect-error tauri does not have sync IPC :(
window.confirm = function (message: string) {
  return invoke("plugin:dialog|confirm", {
    message: message.toString(),
  });
};
