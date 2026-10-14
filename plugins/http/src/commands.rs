// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    collections::HashSet, future::Future, pin::Pin, str::FromStr, sync::Arc, time::Duration,
};

use http::{HeaderMap, HeaderName, HeaderValue, Method, StatusCode, header};
use reqwest::{NoProxy, redirect::Policy};
use serde::{Deserialize, Serialize};
use tauri::{
    Manager, ResourceId, ResourceTable, Runtime, State, Webview,
    async_runtime::Mutex,
    command,
    ipc::{CommandScope, GlobalScope},
};
use tokio::sync::oneshot::{Receiver, Sender, channel};

use crate::{
    Error, Http, RequestContext, Result,
    redirect::{DEFAULT_MAX_REDIRECTIONS, Redirects, send},
    scope::{Entry, Scope},
};

struct ReqwestResponse(reqwest::Response);
impl tauri::Resource for ReqwestResponse {}

type CancelableResponseResult = Result<reqwest::Response>;
type CancelableResponseFuture =
    Pin<Box<dyn Future<Output = CancelableResponseResult> + Send + Sync>>;

struct FetchRequest {
    fut: Mutex<CancelableResponseFuture>,
    abort_tx_rid: ResourceId,
    abort_rx_rid: ResourceId,
}
impl tauri::Resource for FetchRequest {}

struct AbortSender(Sender<()>);
impl tauri::Resource for AbortRecveiver {}

impl AbortSender {
    fn abort(self) {
        let _ = self.0.send(());
    }
}

struct AbortRecveiver(Receiver<()>);
impl tauri::Resource for AbortSender {}

trait AddRequest {
    fn add_request(&mut self, fut: CancelableResponseFuture) -> ResourceId;
}

impl AddRequest for ResourceTable {
    fn add_request(&mut self, fut: CancelableResponseFuture) -> ResourceId {
        let (tx, rx) = channel::<()>();
        let (tx, rx) = (AbortSender(tx), AbortRecveiver(rx));
        let req = FetchRequest {
            fut: Mutex::new(fut),
            abort_tx_rid: self.add(tx),
            abort_rx_rid: self.add(rx),
        };
        self.add(req)
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResponse {
    status: u16,
    status_text: String,
    headers: Vec<(String, String)>,
    url: String,
    rid: ResourceId,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)] //feature flags shoudln't affect api
pub struct DangerousSettings {
    accept_invalid_certs: bool,
    accept_invalid_hostnames: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientConfig {
    method: String,
    url: url::Url,
    headers: Vec<(String, String)>,
    data: Option<Vec<u8>>,
    connect_timeout: Option<u64>,
    max_redirections: Option<usize>,
    proxy: Option<Proxy>,
    danger: Option<DangerousSettings>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Proxy {
    all: Option<UrlOrConfig>,
    http: Option<UrlOrConfig>,
    https: Option<UrlOrConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
pub enum UrlOrConfig {
    Url(String),
    Config(ProxyConfig),
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyConfig {
    url: String,
    basic_auth: Option<BasicAuth>,
    no_proxy: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BasicAuth {
    username: String,
    password: String,
}

#[inline]
fn proxy_creator(
    url_or_config: UrlOrConfig,
    proxy_fn: fn(String) -> reqwest::Result<reqwest::Proxy>,
) -> reqwest::Result<reqwest::Proxy> {
    match url_or_config {
        UrlOrConfig::Url(url) => Ok(proxy_fn(url)?),
        UrlOrConfig::Config(ProxyConfig {
            url,
            basic_auth,
            no_proxy,
        }) => {
            let mut proxy = proxy_fn(url)?;
            if let Some(basic_auth) = basic_auth {
                proxy = proxy.basic_auth(&basic_auth.username, &basic_auth.password);
            }
            if let Some(no_proxy) = no_proxy {
                proxy = proxy.no_proxy(NoProxy::from_string(&no_proxy));
            }
            Ok(proxy)
        }
    }
}

fn attach_proxy(
    proxy: Proxy,
    mut builder: reqwest::ClientBuilder,
) -> crate::Result<reqwest::ClientBuilder> {
    let Proxy { all, http, https } = proxy;

    if let Some(all) = all {
        let proxy = proxy_creator(all, reqwest::Proxy::all)?;
        builder = builder.proxy(proxy);
    }

    if let Some(http) = http {
        let proxy = proxy_creator(http, reqwest::Proxy::http)?;
        builder = builder.proxy(proxy);
    }

    if let Some(https) = https {
        let proxy = proxy_creator(https, reqwest::Proxy::https)?;
        builder = builder.proxy(proxy);
    }

    Ok(builder)
}

#[command]
pub async fn fetch<R: Runtime>(
    webview: Webview<R>,
    state: State<'_, Http>,
    client_config: ClientConfig,
    command_scope: CommandScope<Entry>,
    global_scope: GlobalScope<Entry>,
) -> crate::Result<ResourceId> {
    let ClientConfig {
        method,
        url,
        headers: headers_raw,
        data,
        connect_timeout,
        max_redirections,
        proxy,
        danger,
    } = client_config;

    let scheme = url.scheme();
    let method = Method::from_bytes(method.as_bytes())?;

    let mut headers = HeaderMap::new();
    for (h, v) in headers_raw {
        let name = HeaderName::from_str(&h)?;
        #[cfg(not(feature = "unsafe-headers"))]
        if is_unsafe_header(&name) {
            #[cfg(debug_assertions)]
            {
                eprintln!(
                    "[\x1b[33mWARNING\x1b[0m] Skipping {name} header as it is a forbidden header per fetch spec https://fetch.spec.whatwg.org/#terminology-headers"
                );
                eprintln!(
                    "[\x1b[33mWARNING\x1b[0m] if keeping the header is a desired behavior, you can enable `unsafe-headers` feature flag in your Cargo.toml"
                );
            }
            continue;
        }

        headers.append(name, byte_string_to_header_value(&v)?);
    }

    match scheme {
        "http" | "https" => {
            // taken before the plugin adds its own headers
            let frontend_headers: HashSet<HeaderName> = headers.keys().cloned().collect();

            let scope = Scope::new(
                command_scope
                    .allows()
                    .iter()
                    .chain(global_scope.allows())
                    .cloned()
                    .collect(),
                command_scope
                    .denies()
                    .iter()
                    .chain(global_scope.denies())
                    .cloned()
                    .collect(),
            );

            if !scope.is_allowed(&url) {
                return Err(Error::UrlNotAllowed(url));
            }

            let mut builder = reqwest::ClientBuilder::new();

            if let Some(danger_config) = danger {
                #[cfg(not(feature = "dangerous-settings"))]
                {
                    #[cfg(debug_assertions)]
                    {
                        eprintln!(
                            "[\x1b[33mWARNING\x1b[0m] using dangerous settings requires `dangerous-settings` feature flag in your Cargo.toml"
                        );
                    }
                    let _ = danger_config;
                    return Err(Error::DangerousSettings);
                }
                #[cfg(feature = "dangerous-settings")]
                {
                    builder = builder
                        .danger_accept_invalid_certs(danger_config.accept_invalid_certs)
                        .danger_accept_invalid_hostnames(danger_config.accept_invalid_hostnames)
                }
            }

            if let Some(timeout) = connect_timeout {
                builder = builder.connect_timeout(Duration::from_millis(timeout));
            }

            // followed by `send`, which computes the headers of every hop
            builder = builder.redirect(Policy::none());
            let redirects = Redirects {
                scope: state.config.scope_redirects.then_some(scope),
                max: max_redirections.unwrap_or(DEFAULT_MAX_REDIRECTIONS),
                // the default of `reqwest::ClientBuilder::referer`
                referer: true,
            };

            if let Some(proxy_config) = proxy {
                builder = attach_proxy(proxy_config, builder)?;
            }

            #[cfg(feature = "cookies")]
            {
                builder = builder.cookie_provider(state.cookies_jar.clone());
            }

            // POST and PUT requests should always have a 0 length content-length,
            // if there is no body. https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
            if data.is_none() && matches!(method, Method::POST | Method::PUT) {
                headers.append(header::CONTENT_LENGTH, HeaderValue::from_str("0")?);
            }

            if headers.contains_key(header::RANGE) {
                // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch step 18
                // If httpRequest's header list contains `Range`, then append (`Accept-Encoding`, `identity`)
                headers.append(header::ACCEPT_ENCODING, HeaderValue::from_str("identity")?);
            }

            let webview_origin = webview.url().ok().map(|url| {
                // The url crate returns OpaqueOrigin for tauri://localhost which serializes to "null"
                if url.scheme() == "tauri" {
                    "tauri://localhost".to_string()
                } else {
                    url.origin().ascii_serialization()
                }
            });
            let webview_label = webview.label().to_string();
            let request_headers = state.request_headers.clone();
            let add_headers = move |url: &url::Url,
                                    method: &Method,
                                    frontend_headers: &HashSet<HeaderName>,
                                    headers: &mut HeaderMap| {
                let ctx = RequestContext::new(
                    url,
                    method,
                    &webview_label,
                    webview_origin.as_deref(),
                    frontend_headers,
                );
                request_headers.apply(&ctx, headers)
            };

            let client = builder.build()?;
            let mut request = client.request(method, url);
            if let Some(data) = data {
                request = request.body(data);
            }
            let request = request.build()?;

            let fut = send(
                client,
                request,
                headers,
                frontend_headers,
                redirects,
                add_headers,
            );

            let mut resources_table = webview.resources_table();
            let rid = resources_table.add_request(Box::pin(fut));

            Ok(rid)
        }
        "data" => {
            let data_url =
                data_url::DataUrl::process(url.as_str()).map_err(|_| Error::DataUrlError)?;
            let (body, _) = data_url
                .decode_to_vec()
                .map_err(|_| Error::DataUrlDecodeError)?;

            let response = http::Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, data_url.mime_type().to_string())
                .body(reqwest::Body::from(body))?;

            #[cfg(feature = "tracing")]
            tracing::trace!("{:?}", response);

            let fut = async move { Ok(reqwest::Response::from(response)) };
            let mut resources_table = webview.resources_table();
            let rid = resources_table.add_request(Box::pin(fut));
            Ok(rid)
        }
        _ => Err(Error::SchemeNotSupport(scheme.to_string())),
    }
}

#[command]
pub fn fetch_cancel<R: Runtime>(webview: Webview<R>, rid: ResourceId) -> crate::Result<()> {
    let mut resources_table = webview.resources_table();
    let req = resources_table.get::<FetchRequest>(rid)?;
    let abort_tx = resources_table.take::<AbortSender>(req.abort_tx_rid)?;
    if let Some(abort_tx) = Arc::into_inner(abort_tx) {
        abort_tx.abort();
    }
    Ok(())
}

// Header values are byte strings. Decode as ISO-8859-1 so the raw bytes survive
// as a valid ByteString for `new Headers()` on the JS side.
fn header_value_to_byte_string(value: &HeaderValue) -> String {
    value.as_bytes().iter().map(|&b| b as char).collect()
}

// The inverse of `header_value_to_byte_string`: a ByteString from `Headers` on the
// JS side carries one byte per char, so send those bytes as-is like a browser does.
// Values with chars above U+00FF can't come from `Headers`; keep encoding those as UTF-8.
fn byte_string_to_header_value(value: &str) -> crate::Result<HeaderValue> {
    let bytes = value
        .chars()
        .map(|c| u8::try_from(c).ok())
        .collect::<Option<Vec<u8>>>();
    Ok(match bytes {
        Some(bytes) => HeaderValue::from_bytes(&bytes)?,
        None => HeaderValue::from_str(value)?,
    })
}

#[command]
pub async fn fetch_send<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
) -> crate::Result<FetchResponse> {
    let (req, abort_rx) = {
        let mut resources_table = webview.resources_table();
        let req = resources_table.get::<FetchRequest>(rid)?;
        let abort_rx = resources_table.take::<AbortRecveiver>(req.abort_rx_rid)?;
        (req, abort_rx)
    };

    let Some(abort_rx) = Arc::into_inner(abort_rx) else {
        return Err(Error::RequestCanceled);
    };

    let mut fut = req.fut.lock().await;

    let res = tokio::select! {
        res = fut.as_mut() => res,
        _ = abort_rx.0 => Err(Error::RequestCanceled),
    };

    // Release resources
    {
        let mut resources_table = webview.resources_table();
        let _ = resources_table.take::<AbortSender>(req.abort_tx_rid);
        let _ = resources_table.close(rid);
    }

    let res = res?;

    #[cfg(feature = "tracing")]
    tracing::trace!("{:?}", res);

    let status = res.status();
    let url = res.url().to_string();
    let mut headers = Vec::new();
    for (key, val) in res.headers().iter() {
        headers.push((key.as_str().into(), header_value_to_byte_string(val)));
    }

    let mut resources_table = webview.resources_table();
    let rid = resources_table.add(ReqwestResponse(res));

    Ok(FetchResponse {
        status: status.as_u16(),
        status_text: status.canonical_reason().unwrap_or_default().to_string(),
        headers,
        url,
        rid,
    })
}

#[command]
pub async fn fetch_read_body<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
) -> crate::Result<tauri::ipc::Response> {
    let res = {
        let resources_table = webview.resources_table();
        resources_table.get::<ReqwestResponse>(rid)?
    };

    // SAFETY: we can access the inner value mutably
    // because we are the only ones with a reference to it
    // and we don't want to use `Arc::into_inner` because we want to keep the value in the table
    // for potential future calls to `fetch_cancel_body`
    let res_ptr = Arc::as_ptr(&res) as *mut ReqwestResponse;
    let res = unsafe { &mut *res_ptr };
    let res = &mut res.0;

    let Some(chunk) = res.chunk().await? else {
        let mut resources_table = webview.resources_table();
        resources_table.close(rid)?;

        // return a response with a single byte to indicate that the body is empty
        return Ok(tauri::ipc::Response::new(vec![1]));
    };

    let mut chunk = chunk.to_vec();
    // append a 0 byte to indicate that the body is not empty
    chunk.push(0);

    Ok(tauri::ipc::Response::new(chunk))
}

#[command]
pub async fn fetch_cancel_body<R: Runtime>(
    webview: Webview<R>,
    rid: ResourceId,
) -> crate::Result<()> {
    let mut resources_table = webview.resources_table();
    resources_table.close(rid)?;
    Ok(())
}

// forbidden headers per fetch spec https://fetch.spec.whatwg.org/#terminology-headers
#[cfg(not(feature = "unsafe-headers"))]
fn is_unsafe_header(header: &HeaderName) -> bool {
    matches!(
        *header,
        header::ACCEPT_CHARSET
            | header::ACCEPT_ENCODING
            | header::ACCESS_CONTROL_REQUEST_HEADERS
            | header::ACCESS_CONTROL_REQUEST_METHOD
            | header::CONNECTION
            | header::CONTENT_LENGTH
            | header::COOKIE
            | header::DATE
            | header::DNT
            | header::EXPECT
            | header::HOST
            | header::ORIGIN
            | header::REFERER
            | header::SET_COOKIE
            | header::TE
            | header::TRAILER
            | header::TRANSFER_ENCODING
            | header::UPGRADE
            | header::VIA
    ) || {
        let lower = header.as_str().to_lowercase();
        lower.starts_with("proxy-") || lower.starts_with("sec-")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_byte_string(value: &HeaderValue) {
        let decoded = header_value_to_byte_string(value);
        assert!(
            decoded.chars().all(|c| u32::from(c) <= 255),
            "header value must be a JS ByteString (every code point <= 255)"
        );
        assert_eq!(
            decoded.chars().map(|c| c as u8).collect::<Vec<_>>(),
            value.as_bytes(),
            "ISO-8859-1 decode must preserve the wire bytes"
        );
        let json = serde_json::to_string(&decoded).unwrap();
        let roundtrip: String = serde_json::from_str(&json).unwrap();
        assert_eq!(roundtrip, decoded);
    }

    #[test]
    fn utf8_content_disposition_filename_is_byte_string() {
        // Issue #2898: `filename="中文-EN.pdf"` sent as raw UTF-8 on the wire.
        let value =
            HeaderValue::from_bytes(b"attachment; filename=\"\xe4\xb8\xad\xe6\x96\x87-EN.pdf\"")
                .unwrap();
        assert_byte_string(&value);
    }

    #[test]
    fn non_utf8_header_bytes_are_preserved() {
        let value = HeaderValue::from_bytes(b"x\xff").unwrap();
        assert_byte_string(&value);
    }

    #[test]
    fn request_byte_string_is_sent_as_raw_bytes() {
        let wire = b"attachment; filename=\"\xe4\xb8\xad\xe6\x96\x87-EN.pdf\"";
        let byte_string = header_value_to_byte_string(&HeaderValue::from_bytes(wire).unwrap());
        let value = byte_string_to_header_value(&byte_string).unwrap();
        assert_eq!(value.as_bytes(), wire);

        let value = byte_string_to_header_value("x\u{ff}").unwrap();
        assert_eq!(value.as_bytes(), b"x\xff");
    }

    #[test]
    fn request_non_byte_string_falls_back_to_utf8() {
        let value = byte_string_to_header_value("中文").unwrap();
        assert_eq!(value.as_bytes(), "中文".as_bytes());
    }
}
