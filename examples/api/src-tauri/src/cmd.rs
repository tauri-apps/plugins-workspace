// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Mutex;

use serde::Deserialize;
use tauri::{AppHandle, command};
use tauri_plugin_http::{
    HttpExt, RequestContext,
    reqwest::header::{HeaderMap, HeaderValue, ORIGIN},
};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RequestBody {
    id: i32,
    name: String,
}

#[command]
pub fn log_operation(event: String, payload: Option<String>) {
    log::info!("{} {:?}", event, payload);
}

#[command]
pub fn perform_request(endpoint: String, body: RequestBody) -> String {
    println!("{} {:?}", endpoint, body);
    "message response".into()
}

/// The URLs under `/identity/` the http plugin's `on_request` hook ran for.
static HTTP_HOOK_URLS: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The http plugin's `on_request` hook: requests to `/identity/hook/` get a header from it, and its
/// `Origin` replaces the one from the `requestHeaders` rules, which apply before it.
pub fn http_on_request(ctx: &RequestContext<'_>, headers: &mut HeaderMap) {
    let path = ctx.url().path();
    if path.starts_with("/identity/") {
        HTTP_HOOK_URLS.lock().unwrap().push(ctx.url().to_string());
    }
    if path.starts_with("/identity/hook/") {
        headers.insert(ORIGIN, HeaderValue::from_static("https://hook.example.com"));
        if let Ok(label) = HeaderValue::from_str(ctx.webview_label()) {
            headers.insert("x-hook-webview-label", label);
        }
        if let Some(Ok(origin)) = ctx.webview_origin().map(HeaderValue::from_str) {
            headers.insert("x-hook-webview-origin", origin);
        }
    }
}

#[command]
pub fn http_hook_urls() -> Vec<String> {
    HTTP_HOOK_URLS.lock().unwrap().clone()
}

/// Sends a request from Rust with the http plugin's cookie jar and returns the `Cookie` header the
/// example's echo server received.
#[command]
pub async fn http_native_cookie(app: AppHandle) -> Result<Option<String>, String> {
    let client = tauri_plugin_http::reqwest::Client::builder()
        .cookie_provider(app.http().cookie_jar())
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get("http://localhost:3003/native")
        .send()
        .await
        .map_err(|e| e.to_string())?;
    Ok(response
        .headers()
        .get("cookie")
        .and_then(|value| value.to_str().ok())
        .map(ToString::to_string))
}

#[command]
pub fn http_clear_cookies(app: AppHandle) {
    app.http().cookie_jar().clear();
}
