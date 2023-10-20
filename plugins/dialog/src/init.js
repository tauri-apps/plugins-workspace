// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

window.alert = function (message) {
  invoke("plugin:dialog|message", {
    message: message.toString(),
  });
};

window.confirm = function (message) {
  return invoke("plugin:dialog|confirm", {
    message: message.toString(),
  });
};
