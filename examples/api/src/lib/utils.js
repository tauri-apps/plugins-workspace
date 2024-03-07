// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

export function arrayBufferToBase64(buffer, callback) {
  const blob = new Blob([buffer], {
    type: "application/octet-binary",
  });
  const reader = new FileReader();
  reader.onload = function (evt) {
    const dataurl = evt.target.result;
    callback(dataurl.substr(dataurl.indexOf(",") + 1));
  };
  reader.readAsDataURL(blob);
}
