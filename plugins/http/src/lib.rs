// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Access the HTTP client written in Rust.
//!
//! ## Cargo features
//!
//! ### Reqwest feature forwards
//!
//! These features forwards [`reqwest`](https://docs.rs/reqwest/0.13/reqwest/index.html) features:
//!
//! - **http2** *(enabled by default)*: Enables HTTP/2 support.
//! - **native-tls**: Enables TLS functionality provided by native-tls, including ALPN.
//! - **native-tls-vendored**: Enables the vendored feature of native-tls.
//! - **rustls-tls** *(enabled by default)*: Enables TLS functionality provided by rustls, verifying
//!   certificates with the platform verifier. Uses `ring` as the crypto provider, installed as the
//!   process default only if none is installed yet.
//! - **blocking**: Provides the [blocking](https://docs.rs/reqwest/0.13/reqwest/blocking/index.html) client API.
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
//! - **hickory-dns**: Enables a Hickory DNS async resolver instead of the default threadpool using
//!   getaddrinfo.
//! - **system-proxy** *(enabled by default)*: Use Windows and macOS system proxy settings automatically.
//!
//! ### tauri-plugin-http features
//!
//! - **tracing**: Adds request, response, and cookie-store diagnostics through `tracing`.
//! - **unsafe-headers**: Allows webview requests to send any headers.
//! - **dangerous-settings**: Allows dangerous client settings such as accepting invalid certificates or hostnames.
//!
//! ## Security
//!
//! The URL scope is checked on every hop of a redirect chain, not only on the URL requested by
//! the frontend, so every redirect target must also be allowed by the scope. Otherwise a server on
//! an allowed origin could redirect the request to any other origin - a `localhost` service, an
//! internal host or a cloud metadata endpoint - and hand its response to the webview.

use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
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
    /// Each webview's current page token rid, by webview label.
    pub(crate) pages: std::sync::Mutex<std::collections::HashMap<String, tauri::ResourceId>>,
    #[cfg(feature = "cookies")]
    cookies_jar: std::sync::Arc<crate::reqwest_cookie_store::CookieStoreMutex>,
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("http")
        .setup(|app, _api| {
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
                pages: Default::default(),
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

            #[cfg(not(feature = "cookies"))]
            let _ = (app, event);
        })
        // Cancel a webview's requests and response bodies when its window is destroyed.
        // This listens on each window instead of using the plugin's `on_event`:
        // by the time `on_event` sees `WindowEvent::Destroyed`, Tauri has already
        // removed the window and its webviews from the app, so their resource tables
        // can no longer be looked up. The listener keeps the webview it was registered
        // for, so it also cancels on the right table if the label is reused later.
        //
        // TODO: Also cancel on webview close (a webview closed on its own in a
        // multiwebview window) once Tauri provides a plugin lifecycle hook for it.
        .on_webview_ready(|webview| {
            let webview_ = webview.clone();
            webview.window().on_window_event(move |event| {
                if matches!(event, tauri::WindowEvent::Destroyed) {
                    commands::cancel_all_requests(&webview_);
                    if let Some(state) = webview_.try_state::<Http>() {
                        state.pages.lock().unwrap().remove(webview_.label());
                    }
                }
            });
        })
        .on_page_load(|webview, payload| {
            if matches!(payload.event(), tauri::webview::PageLoadEvent::Started) {
                commands::cancel_all_requests(webview);
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
