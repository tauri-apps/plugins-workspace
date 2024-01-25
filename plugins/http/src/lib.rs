// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/http/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/http)
//!
//! Access the HTTP client written in Rust.

use std::sync::atomic::AtomicU32;
use std::{collections::HashMap, future::Future, pin::Pin};

pub use reqwest;
use reqwest::Response;
use serde::{Deserialize, Deserializer};
use tauri::async_runtime::Mutex;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

pub use error::{Error, Result};

mod commands;
mod error;
mod scope;

type RequestId = u32;
type CancelableResponseResult = Result<Result<reqwest::Response>>;
type CancelableResponseFuture =
    Pin<Box<dyn Future<Output = CancelableResponseResult> + Send + Sync>>;
type RequestTable = HashMap<RequestId, FetchRequest>;
type ResponseTable = HashMap<RequestId, Response>;

struct FetchRequest(Mutex<CancelableResponseFuture>);
impl FetchRequest {
    fn new(f: CancelableResponseFuture) -> Self {
        Self(Mutex::new(f))
    }
}

/// HTTP scope entry object definition.
/// It is a URL that can be accessed by the webview when using the HTTP APIs.
/// The scoped URL is matched against the request URL using a glob pattern.
///
/// Examples:
/// - "https://*" or "https://**" : allows all HTTPS urls
/// - "https://*.github.com/tauri-apps/tauri": allows any subdomain of "github.com" with the "tauri-apps/api" path
/// - "https://myapi.service.com/users/*": allows access to any URLs that begins with "https://myapi.service.com/users/"
#[allow(rustdoc::bare_urls)]
#[derive(Debug)]
pub struct ScopeEntry {
    pub url: glob::Pattern,
}

impl<'de> Deserialize<'de> for ScopeEntry {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ScopeEntryRaw {
            url: String,
        }

        ScopeEntryRaw::deserialize(deserializer).and_then(|raw| {
            Ok(ScopeEntry {
                url: glob::Pattern::new(&raw.url).map_err(|e| {
                    serde::de::Error::custom(format!(
                        "URL `{}` is not a valid glob pattern: {e}",
                        raw.url
                    ))
                })?,
            })
        })
    }
}

struct Http<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    current_id: AtomicU32,
    requests: Mutex<RequestTable>,
    responses: Mutex<ResponseTable>,
}

impl<R: Runtime> Http<R> {
    fn next_id(&self) -> RequestId {
        self.current_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    }
}

trait HttpExt<R: Runtime> {
    fn http(&self) -> &Http<R>;
}

impl<R: Runtime, T: Manager<R>> HttpExt<R> for T {
    fn http(&self) -> &Http<R> {
        self.state::<Http<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::<R>::new("http")
        .js_init_script(include_str!("api-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![
            commands::fetch,
            commands::fetch_cancel,
            commands::fetch_send,
            commands::fetch_read_body,
        ])
        .setup(|app, _api| {
            app.manage(Http {
                app: app.clone(),
                current_id: 0.into(),
                requests: Default::default(),
                responses: Default::default(),
            });
            Ok(())
        })
        .build()
}
