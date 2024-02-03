// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[path = "src/scope_entry.rs"]
mod scope_entry;

const COMMANDS: &[&str] = &["execute", "stdin_write", "kill", "open"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_scope_schema(schemars::schema_for!(scope_entry::Entry))
        .build();
}
