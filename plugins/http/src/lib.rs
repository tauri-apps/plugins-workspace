// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/http/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/http)
//!
//! Access the HTTP client written in Rust.

pub use reqwest;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime
};

pub use error::{Error, Result};

mod commands;
mod error;
mod scope;

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("http")
        .invoke_handler(tauri::generate_handler![
            commands::fetch,
            commands::fetch_cancel,
            commands::fetch_send,
            commands::fetch_read_body,
        ])
        .build()
}
