// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

if (!("os" in window.__TAURI__)) {
  window.__TAURI__.os = {};
}

window.__TAURI__.os.__eol = __TEMPLATE_eol__;
