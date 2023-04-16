// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Runtime,
};

mod commands;
mod error;

use error::Error;

type Result<T> = std::result::Result<T, Error>;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    PluginBuilder::new("fs")
        .invoke_handler(tauri::generate_handler![
            commands::read_file,
            commands::read_text_file,
            commands::write_file,
            commands::read_dir,
            commands::copy_file,
            commands::create_dir,
            commands::remove_dir,
            commands::remove_file,
            commands::rename_file,
            commands::exists,
            commands::metadata
        ])
        .build()
}
