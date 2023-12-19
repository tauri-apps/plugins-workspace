// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc, time::Duration};

use http::{header, HeaderName, HeaderValue, Method, StatusCode};
use reqwest::redirect::Policy;
use serde::Serialize;
use tauri::{async_runtime::Mutex, command, AppHandle, Manager, ResourceId, Runtime};

use crate::{Error, HttpExt, Result};

struct ReqwestResponse(reqwest::Response);

type CancelableResponseResult = Result<Result<reqwest::Response>>;
type CancelableResponseFuture =
    Pin<Box<dyn Future<Output = CancelableResponseResult> + Send + Sync>>;

struct FetchRequest(Mutex<CancelableResponseFuture>);
impl FetchRequest {
    fn new(f: CancelableResponseFuture) -> Self {
        Self(Mutex::new(f))
    }
}

impl tauri::Resource for FetchRequest {}
impl tauri::Resource for ReqwestResponse {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FetchResponse {
    status: u16,
    status_text: String,
    headers: Vec<(String, String)>,
    url: String,
    rid: ResourceId,
}

#[command]
pub async fn fetch<R: Runtime>(
    app: AppHandle<R>,
    method: String,
    url: url::Url,
    headers: Vec<(String, String)>,
    data: Option<Vec<u8>>,
    connect_timeout: Option<u64>,
    max_redirections: Option<usize>,
) -> crate::Result<ResourceId> {
    let scheme = url.scheme();
    let method = Method::from_bytes(method.as_bytes())?;
    let headers: HashMap<String, String> = HashMap::from_iter(headers);

    match scheme {
        "http" | "https" => {
            if app.http().scope.is_allowed(&url) {
                let mut builder = reqwest::ClientBuilder::new();

                if let Some(timeout) = connect_timeout {
                    builder = builder.connect_timeout(Duration::from_millis(timeout));
                }

                if let Some(max_redirections) = max_redirections {
                    builder = builder.redirect(if max_redirections == 0 {
                        Policy::none()
                    } else {
                        Policy::limited(max_redirections)
                    });
                }

                let mut request = builder.build()?.request(method.clone(), url);

                for (key, value) in &headers {
                    let name = HeaderName::from_bytes(key.as_bytes())?;
                    let v = HeaderValue::from_bytes(value.as_bytes())?;
                    if !matches!(name, header::HOST | header::CONTENT_LENGTH) {
                        request = request.header(name, v);
                    }
                }

                // POST and PUT requests should always have a 0 length content-length,
                // if there is no body. https://fetch.spec.whatwg.org/#http-network-or-cache-fetch
                if data.is_none() && matches!(method, Method::POST | Method::PUT) {
                    request = request.header(header::CONTENT_LENGTH, HeaderValue::from(0));
                }

                if headers.contains_key(header::RANGE.as_str()) {
                    // https://fetch.spec.whatwg.org/#http-network-or-cache-fetch step 18
                    // If httpRequest’s header list contains `Range`, then append (`Accept-Encoding`, `identity`)
                    request = request.header(
                        header::ACCEPT_ENCODING,
                        HeaderValue::from_static("identity"),
                    );
                }

                if !headers.contains_key(header::USER_AGENT.as_str()) {
                    request = request.header(header::USER_AGENT, HeaderValue::from_static("tauri"));
                }

                if let Some(data) = data {
                    request = request.body(data);
                }

                let fut = async move { Ok(request.send().await.map_err(Into::into)) };
                let mut resources_table = app.resources_table();
                let rid = resources_table.add(FetchRequest::new(Box::pin(fut)));

                Ok(rid)
            } else {
                Err(Error::UrlNotAllowed(url))
            }
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

            let fut = async move { Ok(Ok(reqwest::Response::from(response))) };
            let mut resources_table = app.resources_table();
            let rid = resources_table.add(FetchRequest::new(Box::pin(fut)));
            Ok(rid)
        }
        _ => Err(Error::SchemeNotSupport(scheme.to_string())),
    }
}

#[command]
pub async fn fetch_cancel<R: Runtime>(app: AppHandle<R>, rid: ResourceId) -> crate::Result<()> {
    let req = {
        let resources_table = app.resources_table();
        resources_table.get::<FetchRequest>(rid)?
    };
    let mut req = req.0.lock().await;
    *req = Box::pin(async { Err(Error::RequestCanceled) });
    Ok(())
}

#[command]
pub async fn fetch_send<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
) -> crate::Result<FetchResponse> {
    let req = {
        let mut resources_table = app.resources_table();
        resources_table.take::<FetchRequest>(rid)?
    };

    let res = match req.0.lock().await.as_mut().await {
        Ok(Ok(res)) => res,
        Ok(Err(e)) | Err(e) => return Err(e),
    };

    let status = res.status();
    let url = res.url().to_string();
    let mut headers = Vec::new();
    for (key, val) in res.headers().iter() {
        headers.push((
            key.as_str().into(),
            String::from_utf8(val.as_bytes().to_vec())?,
        ));
    }

    let mut resources_table = app.resources_table();
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
pub(crate) async fn fetch_read_body<R: Runtime>(
    app: AppHandle<R>,
    rid: ResourceId,
) -> crate::Result<tauri::ipc::Response> {
    let res = {
        let mut resources_table = app.resources_table();
        resources_table.take::<ReqwestResponse>(rid)?
    };
    let res = Arc::into_inner(res).unwrap().0;
    Ok(tauri::ipc::Response::new(res.bytes().await?.to_vec()))
}
