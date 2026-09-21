// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{future::Future, pin::Pin, str::FromStr, sync::Arc, time::Duration};

use http::{header, HeaderMap, HeaderName, HeaderValue, Method, StatusCode};
use reqwest::{redirect::Policy, NoProxy};
use serde::{Deserialize, Serialize};
use tauri::{
    async_runtime::Mutex,
    command,
    ipc::{CommandScope, GlobalScope},
    Manager, ResourceId, ResourceTable, Runtime, State, Webview,
};
use tokio::sync::oneshot::{channel, Receiver, Sender};

use crate::{
    scope::{Entry, Scope},
    Error, Http, Result,
};

const HTTP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);

/// Matches the default [`reqwest`] redirect policy, used when the frontend does not
/// configure `maxRedirections`.
const DEFAULT_MAX_REDIRECTIONS: usize = 10;

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

/// Builds the redirect policy for the request.
///
/// **Every** hop is checked against the scope. Validating only the URL the frontend asked for is
/// not enough: a server on an allowed origin can answer with a redirect to any other origin (an
/// open redirect, or a server the attacker controls), and following it would give the webview
/// access to a URL the scope denies.
fn redirect_policy(scope: Scope, max_redirections: Option<usize>) -> Policy {
    if max_redirections == Some(0) {
        return Policy::none();
    }

    let max_redirections = max_redirections.unwrap_or(DEFAULT_MAX_REDIRECTIONS);

    Policy::custom(move |attempt| {
        // the first URL in `previous` is the initial request, so it must be excluded
        if attempt.previous().len() > max_redirections {
            attempt.error("too many redirects")
        } else if scope.is_allowed(attempt.url()) {
            attempt.follow()
        } else {
            let url = attempt.url().clone();
            attempt.error(Error::UrlNotAllowed(url))
        }
    })
}

/// [`reqwest`] wraps the error returned by the redirect policy, and its `Display` impl does not
/// include the source, so we unwrap our own scope error to keep the reason visible to the frontend.
fn map_request_error(error: reqwest::Error) -> Error {
    if error.is_redirect() {
        let mut source = std::error::Error::source(&error);
        while let Some(err) = source {
            if let Some(Error::UrlNotAllowed(url)) = err.downcast_ref::<Error>() {
                return Error::UrlNotAllowed(url.clone());
            }
            source = err.source();
        }
    }

    Error::Network(error)
}

// `state` is only read when the `cookies` feature is enabled
#[cfg_attr(not(feature = "cookies"), allow(unused_variables))]
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
                eprintln!("[\x1b[33mWARNING\x1b[0m] Skipping {name} header as it is a forbidden header per fetch spec https://fetch.spec.whatwg.org/#terminology-headers");
                eprintln!("[\x1b[33mWARNING\x1b[0m] if keeping the header is a desired behavior, you can enable `unsafe-headers` feature flag in your Cargo.toml");
            }
            continue;
        }

        headers.append(name, HeaderValue::from_str(&v)?);
    }

    match scheme {
        "http" | "https" => {
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
                        eprintln!("[\x1b[33mWARNING\x1b[0m] using dangerous settings requires `dangerous-settings` feature flag in your Cargo.toml");
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

            builder = builder.redirect(redirect_policy(scope, max_redirections));

            if let Some(proxy_config) = proxy {
                builder = attach_proxy(proxy_config, builder)?;
            }

            #[cfg(feature = "cookies")]
            {
                builder = builder.cookie_provider(state.cookies_jar.clone());
            }

            let mut request = builder.build()?.request(method.clone(), url);

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

            if !headers.contains_key(header::USER_AGENT) {
                headers.append(header::USER_AGENT, HeaderValue::from_str(HTTP_USER_AGENT)?);
            }

            // ensure we have an Origin header set
            if cfg!(not(feature = "unsafe-headers")) || !headers.contains_key(header::ORIGIN) {
                if let Ok(url) = webview.url() {
                    // The url crate returns OpaqueOrigin for tauri://localhost which serializes to "null"
                    let origin = if url.scheme() == "tauri" {
                        "tauri://localhost".to_string()
                    } else {
                        url.origin().ascii_serialization()
                    };
                    headers.append(header::ORIGIN, HeaderValue::from_str(&origin)?);
                }
            }

            // In case empty origin is passed, remove it. Some services do not like Origin header
            // so this way we can remove it in explicit way. The default behaviour is still to set it
            if cfg!(feature = "unsafe-headers")
                && headers.get(header::ORIGIN) == Some(&HeaderValue::from_static(""))
            {
                headers.remove(header::ORIGIN);
            };

            if let Some(data) = data {
                request = request.body(data);
            }

            request = request.headers(headers);

            #[cfg(feature = "tracing")]
            tracing::trace!("{:?}", request);

            let fut = async move { request.send().await.map_err(map_request_error) };

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
        res = fut.as_mut() => res?,
        _ = abort_rx.0 => {
            let mut resources_table = webview.resources_table();
            resources_table.close(rid)?;
            return Err(Error::RequestCanceled);
        }
    };

    #[cfg(feature = "tracing")]
    tracing::trace!("{:?}", res);

    let status = res.status();
    let url = res.url().to_string();
    let mut headers = Vec::new();
    for (key, val) in res.headers().iter() {
        headers.push((
            key.as_str().into(),
            String::from_utf8(val.as_bytes().to_vec())?,
        ));
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
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::Arc,
    };

    use super::*;

    /// Test server with an open redirect:
    ///
    /// - `/redirect-external` answers a 302 to `http://127.0.0.1:{port}/secret`, which is a
    ///   different origin as far as the scope is concerned - the tests allow `localhost`;
    /// - `/redirect/{n}` answers a 302 to `/redirect/{n-1}` on `localhost`, and `/redirect/0`
    ///   answers a 302 to `/target`, so the whole chain stays in scope;
    /// - anything else answers a 200, so a bypass shows up as a successful response.
    fn spawn_server() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();

                // the whole request must be consumed, otherwise closing the connection
                // with unread data queued resets it before the client reads the response
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                loop {
                    let mut header = String::new();
                    reader.read_line(&mut header).unwrap();
                    if matches!(header.as_str(), "" | "\r\n" | "\n") {
                        break;
                    }
                }

                let path = request_line.split(' ').nth(1).unwrap_or("/");
                let location = match path.strip_prefix("/redirect/") {
                    Some("0") => Some(format!("http://localhost:{port}/target")),
                    Some(remaining) => Some(format!(
                        "http://localhost:{port}/redirect/{}",
                        remaining.parse::<usize>().unwrap() - 1
                    )),
                    None if path == "/redirect-external" => {
                        Some(format!("http://127.0.0.1:{port}/secret"))
                    }
                    None => None,
                };

                let response = match location {
                    Some(location) => format!(
                        "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    ),
                    None => "HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nsecret"
                        .to_string(),
                };

                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        port
    }

    fn get(url: &str, scope: Scope, max_redirections: Option<usize>) -> Result<reqwest::Response> {
        let client = reqwest::ClientBuilder::new()
            .redirect(redirect_policy(scope, max_redirections))
            .build()
            .unwrap();
        let request = client.get(url);
        tauri::async_runtime::block_on(
            async move { request.send().await.map_err(map_request_error) },
        )
    }

    fn localhost_scope(port: u16) -> Scope {
        let entry = Arc::new(format!("http://localhost:{port}/*").parse().unwrap());
        Scope::new(vec![entry], Vec::new())
    }

    #[test]
    fn redirect_outside_of_scope_is_denied() {
        let port = spawn_server();

        let err = get(
            &format!("http://localhost:{port}/redirect-external"),
            localhost_scope(port),
            None,
        )
        .unwrap_err();

        match err {
            Error::UrlNotAllowed(url) => {
                assert_eq!(url.as_str(), format!("http://127.0.0.1:{port}/secret"))
            }
            e => panic!("expected the redirect to be denied by the scope, got {e:?}"),
        }
    }

    #[test]
    fn redirect_denied_by_the_scope_is_denied() {
        let port = spawn_server();
        let allow = Arc::new(format!("http://localhost:{port}/*").parse().unwrap());
        let deny = Arc::new(format!("http://localhost:{port}/target").parse().unwrap());
        let scope = Scope::new(vec![allow], vec![deny]);

        let err = get(&format!("http://localhost:{port}/redirect/0"), scope, None).unwrap_err();

        assert!(matches!(err, Error::UrlNotAllowed(_)));
    }

    #[test]
    fn redirect_inside_of_scope_is_followed() {
        let port = spawn_server();

        let response = get(
            &format!("http://localhost:{port}/redirect/2"),
            localhost_scope(port),
            None,
        )
        .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.url().path(), "/target");
    }

    #[test]
    fn max_redirections_is_enforced() {
        let port = spawn_server();

        let err = get(
            &format!("http://localhost:{port}/redirect/5"),
            localhost_scope(port),
            Some(2),
        )
        .unwrap_err();

        assert!(matches!(err, Error::Network(e) if e.is_redirect()));
    }

    #[test]
    fn zero_max_redirections_does_not_follow() {
        let port = spawn_server();

        let response = get(
            &format!("http://localhost:{port}/redirect/0"),
            localhost_scope(port),
            Some(0),
        )
        .unwrap();

        assert_eq!(response.status(), StatusCode::FOUND);
    }
}
