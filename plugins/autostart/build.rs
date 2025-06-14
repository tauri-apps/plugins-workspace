// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &["enable", "disable", "is_enabled"];

fn main() {
    let result = tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .try_build();

    // - FIXME: Temporarily ignore writing errors on docs.rs, this is a mitigation for <https://github.com/tauri-apps/tauri/pull/13597#issuecomment-2961321899>
    if !cfg!(docsrs) {
        result.unwrap();
    }
}
