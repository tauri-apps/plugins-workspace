// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

window.addEventListener("DOMContentLoaded", () => {
  document.querySelector("#greet-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
  });
});
