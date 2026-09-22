// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Expose your apps assets through a localhost server instead of the default custom protocol.
//!
//! **Note: This plugins brings considerable security risks and you should only use it if you know what your are doing. If in doubt, use the default custom protocol implementation.**

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use std::collections::HashMap;

use http::Uri;
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Runtime,
};
use tiny_http::{Header, Response as HttpResponse, Server};

/// An incoming HTTP request received by the localhost server, passed to the
/// [`Builder::on_request`] hook.
pub struct Request {
    url: String,
}

impl Request {
    /// The request URL (path and, if present, query string), exactly as sent by the client.
    pub fn url(&self) -> &str {
        &self.url
    }
}

/// The HTTP response the localhost server is about to send back for a matched asset. Passed
/// mutably to the [`Builder::on_request`] hook so it can add or override headers before the
/// response is written to the client.
pub struct Response {
    headers: HashMap<String, String>,
}

impl Response {
    /// Adds a header to the response, replacing any existing header with the same name.
    pub fn add_header<H: Into<String>, V: Into<String>>(&mut self, header: H, value: V) {
        self.headers.insert(header.into(), value.into());
    }
}

type OnRequest = Option<Box<dyn Fn(&Request, &mut Response) + Send + Sync>>;

/// Builds the localhost plugin.
///
/// **Note: This plugin brings considerable security risks and you should only use it if you know
/// what you are doing. Because the server has no authentication, any local process can connect to
/// it and read the assets it serves. If in doubt, use the default custom protocol implementation.**
pub struct Builder {
    port: u16,
    host: Option<String>,
    on_request: OnRequest,
}

impl Builder {
    /// Creates a new [`Builder`] that will serve the app's assets on the given `port`, bound to
    /// `localhost` unless [`Self::host`] is called.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// fn setup<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
    ///     builder.plugin(
    ///         tauri_plugin_localhost::Builder::new(9527)
    ///             .host("127.0.0.1")
    ///             .on_request(|request, response| {
    ///                 println!("{}", request.url());
    ///             })
    ///             .build(),
    ///     )
    /// }
    /// ```
    pub fn new(port: u16) -> Self {
        Self {
            port,
            host: None,
            on_request: None,
        }
    }

    /// Sets the host the localhost server binds to. Defaults to `localhost`.
    pub fn host<H: Into<String>>(mut self, host: H) -> Self {
        self.host = Some(host.into());
        self
    }

    /// Sets a hook that is called for every request that resolves to a known frontend asset,
    /// right before the response is sent. Use it to inspect the [`Request`] and add or override
    /// headers on the [`Response`], for example to append custom CORS or caching headers.
    pub fn on_request<F: Fn(&Request, &mut Response) + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.on_request.replace(Box::new(f));
        self
    }

    /// Builds the plugin, ready to be registered with [`tauri::Builder::plugin`].
    ///
    /// On setup it spawns a background thread that starts a `tiny_http` server bound to
    /// `host:port`. For every incoming request whose path resolves to a known frontend asset, the
    /// server responds with that asset's bytes, automatically setting the `Content-Type`, the
    /// `Content-Security-Policy` (when the asset has one) and a `Cache-Control: no-cache` header,
    /// then invoking the [`Self::on_request`] hook (if any) before writing the response.
    ///
    /// # Panics
    ///
    /// Panics on the background thread if the server fails to bind to `host:port`, or if it fails
    /// to send a response for a request.
    pub fn build<R: Runtime>(mut self) -> TauriPlugin<R> {
        let port = self.port;
        let host = self.host.unwrap_or("localhost".to_string());
        let on_request = self.on_request.take();

        PluginBuilder::new("localhost")
            .setup(move |app, _api| {
                let asset_resolver = app.asset_resolver();
                std::thread::spawn(move || {
                    let server =
                        Server::http(format!("{host}:{port}")).expect("Unable to spawn server");
                    for req in server.incoming_requests() {
                        let path = req
                            .url()
                            .parse::<Uri>()
                            .map(|uri| uri.path().into())
                            .unwrap_or_else(|_| req.url().into());

                        #[allow(unused_mut)]
                        if let Some(mut asset) = asset_resolver.get(path) {
                            let request = Request {
                                url: req.url().into(),
                            };
                            let mut response = Response {
                                headers: Default::default(),
                            };

                            response.add_header("Content-Type", asset.mime_type);
                            if let Some(csp) = asset.csp_header {
                                response
                                    .headers
                                    .insert("Content-Security-Policy".into(), csp);
                            }

                            response
                                .headers
                                .insert("Cache-Control".into(), "no-cache".into());

                            if let Some(on_request) = &on_request {
                                on_request(&request, &mut response);
                            }

                            let mut resp = HttpResponse::from_data(asset.bytes);
                            for (header, value) in response.headers {
                                if let Ok(h) = Header::from_bytes(header.as_bytes(), value) {
                                    resp.add_header(h);
                                }
                            }
                            req.respond(resp).expect("unable to setup response");
                        }
                    }
                });
                Ok(())
            })
            .build()
    }
}
