// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &[
    "notify",
    "request_permission",
    "is_permission_granted",
    "register_action_types",
    "register_listener",
    "cancel",
    "get_pending",
    "remove_active",
    "get_active",
    "check_permissions",
    "show",
    "batch",
    "list_channels",
    "delete_channel",
    "create_channel",
    "permission_state"
];

fn main() {
    if let Err(error) = tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .android_path("android")
        .ios_path("ios")
        .try_build()
    {
        println!("{error:#}");
        // when building documentation for Android the plugin build result is irrelevant to the crate itself
        if !(cfg!(docsrs) && std::env::var("TARGET").unwrap().contains("android")) {
            std::process::exit(1);
        }
    }
}
