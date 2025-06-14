// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &[
    "initialize",
    "destroy",
    "save",
    "create_client",
    "load_client",
    "get_store_record",
    "save_store_record",
    "remove_store_record",
    "save_secret",
    "remove_secret",
    "execute_procedure",
];

fn main() {
    let result = tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .try_build();

    // - FIXME: Temporarily ignore writing errors on docs.rs, this is a mitigation for <https://github.com/tauri-apps/tauri/pull/13597#issuecomment-2961321899>
    if !cfg!(docsrs) {
        result.unwrap();
    }
}
