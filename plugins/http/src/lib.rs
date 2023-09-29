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
use tauri::async_runtime::Mutex;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

use crate::config::{Config, HttpAllowlistScope};
pub use error::{Error, Result};

mod commands;
mod config;
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

struct Http<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    scope: scope::Scope,
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

pub fn init<R: Runtime>() -> TauriPlugin<R, Option<Config>> {
    Builder::<R, Option<Config>>::new("http")
        .js_init_script(include_str!("api-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![
            commands::fetch,
            commands::fetch_cancel,
            commands::fetch_send,
            commands::fetch_read_body,
        ])
        .setup(|app, api| {
            let default_scope = HttpAllowlistScope::default();
            app.manage(Http {
                app: app.clone(),
                current_id: 0.into(),
                requests: Default::default(),
                responses: Default::default(),
                scope: scope::Scope::new(
                    api.config()
                        .as_ref()
                        .map(|c| &c.scope)
                        .unwrap_or(&default_scope),
                ),
            });
            Ok(())
        })
        .build()
}
