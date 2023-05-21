// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("process")
        .js_init_script(include_str!("api-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![commands::exit, commands::restart])
        .build()
}
