// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Access the HTTP client written in Rust.

use std::io::Cursor;

pub use reqwest;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use error::{Error, Result};

mod commands;
mod error;
#[cfg(feature = "cookies")]
mod reqwest_cookie_store;
mod scope;

pub(crate) struct Http {
    #[cfg(feature = "cookies")]
    cookies_jar_path: std::path::PathBuf,
    #[cfg(feature = "cookies")]
    cookies_jar: std::sync::Arc<crate::reqwest_cookie_store::CookieStoreMutex>,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("http")
        .setup(|app, _| {
            #[cfg(feature = "cookies")]
            let (cookies_jar_path, cookies_jar) = {
                use crate::reqwest_cookie_store::*;
                use std::fs::File;
                use std::io::BufReader;
                use std::sync::Arc;

                let cache_dir = app.path().app_cache_dir()?;
                std::fs::create_dir_all(&cache_dir)?;

                let path = cache_dir.join("Cookies");
                let file = File::options()
                    .create(true)
                    .append(true)
                    .read(true)
                    .open(&path)?;

                let reader = BufReader::new(file);
                let store = CookieStoreMutex::load(reader)
                    .or_else(|_e| {
                        #[cfg(feature = "tracing")]
                        tracing::warn!(
                            "failed to load cookie store: {_e}, falling back to empty store"
                        );
                        CookieStoreMutex::load(Cursor::new("[]".to_string()))
                    })
                    .map_err(|e| e.to_string())?;

                (path, Arc::new(store))
            };

            let state = Http {
                #[cfg(feature = "cookies")]
                cookies_jar_path,
                #[cfg(feature = "cookies")]
                cookies_jar,
            };

            app.manage(state);

            Ok(())
        })
        .on_event(|app, event| {
            #[cfg(feature = "cookies")]
            if let tauri::RunEvent::Exit = event {
                use std::fs::File;
                use std::io::BufWriter;

                let state = app.state::<Http>();

                if let Ok(file) = File::create(&state.cookies_jar_path) {
                    let mut writer = BufWriter::new(file);
                    let _ = state.cookies_jar.save(&mut writer);
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::fetch,
            commands::fetch_cancel,
            commands::fetch_send,
            commands::fetch_read_body
        ])
        .build()
}
