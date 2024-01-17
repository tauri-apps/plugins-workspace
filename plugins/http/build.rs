// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

const COMMANDS: &[&str] = &["fetch", "fetch_cancel", "fetch_send", "fetch_read_body"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
