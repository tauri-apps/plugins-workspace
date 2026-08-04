// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Access the HTTP client written in Rust.
//!
//! ## Cargo features
//!
//! - **multipart**: Adds support for multipart form bodies.
//! - **json**: Adds support for JSON request and response helpers.
//! - **stream**: Adds support for streaming bodies.
//! - **blocking**: Adds support for the blocking `reqwest` client API; plugin commands remain async.
//! - **cookies** *(enabled by default)*: Adds support for cookie handling and a persistent cookie store.
//! - **http2** *(enabled by default)*: Adds support for HTTP/2.
//! - **charset** *(enabled by default)*: Adds support for decoding non-UTF-8 response character sets.
//! - **rustls-tls** *(enabled by default)*: Adds TLS support through Rustls with its standard root configuration.
//! - **rustls-tls-manual-roots**: Adds TLS support through Rustls without an automatically installed root
//!   source.
//! - **rustls-tls-webpki-roots**: Adds TLS support through Rustls with Mozilla/WebPKI roots.
//! - **rustls-tls-native-roots**: Adds TLS support through Rustls with the platform's native root store.
//! - **native-tls**: Adds TLS support through the platform-native TLS backend.
//! - **native-tls-vendored**: Adds native TLS support with vendored OpenSSL where applicable.
//! - **native-tls-alpn**: Adds ALPN support to the native TLS backend.
//! - **trust-dns**: Adds support for the trust-dns/Hickory DNS resolver.
//! - **socks**: Adds support for SOCKS proxies.
//! - **macos-system-configuration** *(enabled by default)*: Adds macOS System Configuration proxy
//!   discovery.
//! - **gzip**: Adds support for automatic gzip response decompression.
//! - **brotli**: Adds support for automatic Brotli response decompression.
//! - **deflate**: Adds support for automatic deflate response decompression.
//! - **zstd**: Adds support for automatic Zstandard response decompression.
//! - **tracing**: Adds request, response, and cookie-store diagnostics through `tracing`.
//! - **unsafe-headers**: Allows frontend requests to retain headers normally
//!   stripped or controlled by the plugin.
//! - **dangerous-settings**: Allows dangerous client settings such as accepting
//!   invalid certificates or hostnames.
//!
//! Most features forward to the same-named `reqwest` feature.

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

#[cfg(feature = "cookies")]
const COOKIES_FILENAME: &str = ".cookies";

pub(crate) struct Http {
    #[cfg(feature = "cookies")]
    cookies_jar: std::sync::Arc<crate::reqwest_cookie_store::CookieStoreMutex>,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("http")
        .setup(|app, _| {
            #[cfg(feature = "cookies")]
            let cookies_jar = {
                use crate::reqwest_cookie_store::*;
                use std::fs::File;
                use std::io::BufReader;

                let cache_dir = app.path().app_cache_dir()?;
                std::fs::create_dir_all(&cache_dir)?;

                let path = cache_dir.join(COOKIES_FILENAME);
                let file = File::options()
                    .create(true)
                    .append(true)
                    .read(true)
                    .open(&path)?;

                let reader = BufReader::new(file);
                CookieStoreMutex::load(path.clone(), reader).unwrap_or_else(|_e| {
                    #[cfg(feature = "tracing")]
                    tracing::warn!(
                        "failed to load cookie store: {_e}, falling back to empty store"
                    );
                    CookieStoreMutex::new(path, Default::default())
                })
            };

            let state = Http {
                #[cfg(feature = "cookies")]
                cookies_jar: std::sync::Arc::new(cookies_jar),
            };

            app.manage(state);

            Ok(())
        })
        .on_event(|app, event| {
            #[cfg(feature = "cookies")]
            if let tauri::RunEvent::Exit = event {
                let state = app.state::<Http>();

                match state.cookies_jar.request_save() {
                    Ok(rx) => {
                        let _ = rx.recv();
                    }
                    Err(_e) => {
                        #[cfg(feature = "tracing")]
                        tracing::error!("failed to save cookie jar: {_e}");
                    }
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::fetch,
            commands::fetch_cancel,
            commands::fetch_send,
            commands::fetch_read_body,
            commands::fetch_cancel_body,
        ])
        .build()
}
