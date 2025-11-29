// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod commands;
mod error;

use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("recent-doc")
    .invoke_handler(tauri::generate_handler![
      commands::add_recent_document,
      commands::clear_recent_documents,
      commands::get_recent_documents
    ])
    .build()
}
