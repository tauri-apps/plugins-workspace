// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Access the HTTP client written in Rust.
//!
//! ## Cargo features
//!
//! ### Reqwest feature forwards
//!
//! These features forwards [`reqwest`](https://docs.rs/reqwest/0.12.28/reqwest/index.html) features:
//!
//! - **http2** *(enabled by default)*: Enables HTTP/2 support.
//! - **native-tls**: Enables TLS functionality provided by native-tls.
//! - **native-tls-vendored**: Enables the vendored feature of native-tls.
//! - **native-tls-alpn**: Enables the alpn feature of native-tls.
//! - **rustls-tls** *(enabled by default)*: Enables TLS functionality provided by rustls. Equivalent to
//!   rustls-tls-webpki-roots.
//! - **rustls-tls-manual-roots**: Enables TLS functionality provided by rustls, without setting any root
//!   certificates. Roots have to be specified manually.
//! - **rustls-tls-webpki-roots**: Enables TLS functionality provided by rustls, while using root certificates
//!   from the webpki-roots crate.
//! - **rustls-tls-native-roots**: Enables TLS functionality provided by rustls, while using root certificates
//!   from the rustls-native-certs crate.
//! - **blocking**: Provides the [blocking](https://docs.rs/reqwest/0.12.28/reqwest/blocking/index.html) client API.
//! - **charset** *(enabled by default)*: Improved support for decoding text.
//! - **cookies** *(enabled by default)*: Provides cookie session support.
//! - **gzip**: Provides response body gzip decompression.
//! - **brotli**: Provides response body brotli decompression.
//! - **zstd**: Provides response body zstd decompression.
//! - **deflate**: Provides response body deflate decompression.
//! - **json**: Provides serialization and deserialization for JSON bodies.
//! - **multipart**: Provides functionality for multipart forms.
//! - **stream**: Adds support for futures::Stream.
//! - **socks**: Provides SOCKS5 proxy support.
//! - **trust-dns**: Enables a trust-dns/Hickory DNS async resolver instead of the default threadpool using
//!   getaddrinfo.
//! - **macos-system-configuration** *(deprecated, use `system-proxy` instead)*: Use Windows and macOS system proxy settings automatically.
//! - **system-proxy** *(enabled by default)*: Use Windows and macOS system proxy settings automatically.
//!
//! ### tauri-plugin-http features
//!
//! - **tracing**: Adds request, response, and cookie-store diagnostics through `tracing`.
//! - **unsafe-headers**: Allows webview requests to send any headers.
//! - **dangerous-settings**: Allows dangerous client settings such as accepting invalid certificates or hostnames.

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
