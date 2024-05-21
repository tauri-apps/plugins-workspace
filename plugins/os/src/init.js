// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// eslint-disable-next-line
Object.defineProperty(window, "__TAURI_OS_PLUGIN_INTERNALS__", {
  value: {
    eol: __TEMPLATE_eol__,
    os_type: __TEMPLATE_os_type__,
    platform: __TEMPLATE_platform__,
    family: __TEMPLATE_family__,
    version: __TEMPLATE_version__,
    arch: __TEMPLATE_arch__,
    exe_extension: __TEMPLATE_exe_extension__,
  },
});
