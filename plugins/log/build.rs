// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::process::exit;

const COMMANDS: &[&str] = &["log"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();

    if let Err(error) = tauri_build::mobile::PluginBuilder::new()
        .ios_path("ios")
        .run()
    {
        println!("{error:#}");
        exit(1);
    }
}
