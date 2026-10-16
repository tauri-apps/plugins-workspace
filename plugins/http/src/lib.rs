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
//!
//! ## Configuration
//!
//! See [`Config`] for the options that can be set on the `plugins > http` object of your
//! `tauri.conf.json`:
//!
//! ```json
//! {
//!   "plugins": {
//!     "http": {
//!       "scopeRedirects": true,
//!       "userAgent": "my-app/1.0",
//!       "requestHeaders": [
//!         {
//!           "url": "https://api.example.com/*",
//!           "headers": { "Origin": "https://desktop.example.com" }
//!         }
//!       ]
//!     }
//!   }
//! }
//! ```
//!
//! The same options, and an [`on_request`](Builder::on_request) hook for policies the configuration
//! cannot express, can be set in Rust with the plugin [`Builder`].

pub use reqwest;
use reqwest::header::HeaderMap;
use tauri::{
    Manager, Runtime,
    plugin::{Builder as PluginBuilder, TauriPlugin},
};

pub use config::Config;
pub use error::{Error, Result};
pub use request_headers::{HeaderRuleMode, RequestContext, RequestHeaderRule};
#[cfg(feature = "cookies")]
pub use reqwest_cookie_store::CookieJar;

mod commands;
mod config;
mod error;
mod redirect;
mod request_headers;
#[cfg(feature = "cookies")]
mod reqwest_cookie_store;
mod scope;

#[cfg(feature = "cookies")]
const COOKIES_FILENAME: &str = ".cookies";

/// Access to the plugin's state, see [`HttpExt`].
pub struct Http {
    pub(crate) config: Config,
    pub(crate) request_headers: std::sync::Arc<request_headers::RequestHeaders>,
    #[cfg(feature = "cookies")]
    cookies_jar: std::sync::Arc<CookieJar>,
}

impl Http {
    /// The cookie jar used by every request sent from the webview.
    ///
    /// Give it to your own [`reqwest`] clients to share the webview's session with them; see
    /// [`CookieJar`].
    #[cfg(feature = "cookies")]
    pub fn cookie_jar(&self) -> std::sync::Arc<CookieJar> {
        self.cookies_jar.clone()
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the HTTP
/// plugin.
pub trait HttpExt<R: Runtime> {
    /// The plugin's state.
    ///
    /// # Panics
    ///
    /// Panics if the plugin is not registered.
    fn http(&self) -> &Http;
}

impl<R: Runtime, T: Manager<R>> HttpExt<R> for T {
    fn http(&self) -> &Http {
        self.state::<Http>().inner()
    }
}

/// Initializes the plugin with the default options, the same as `Builder::new().build()`.
///
/// The plugin reads its [`Config`] from the `plugins > http` object of the `tauri.conf.json` file;
/// when that object is missing, [`Config::default`] is used.
///
/// With the `cookies` Cargo feature (enabled by default), a cookie jar is loaded from a `.cookies`
/// file in the application cache directory on setup and written back to it when the application
/// exits. A jar that cannot be read is replaced by an empty one.
///
/// Register it on the Tauri builder with `.plugin(tauri_plugin_http::init())`.
pub fn init<R: Runtime>() -> TauriPlugin<R, Option<Config>> {
    Builder::new().build()
}

/// The plugin builder.
///
/// Options set here take precedence over the same options in the [`Config`].
///
/// ```rust,no_run
/// use tauri_plugin_http::reqwest::header::{HeaderValue, ORIGIN};
///
/// # fn f<R: tauri::Runtime>(builder: tauri::Builder<R>) -> tauri::Builder<R> {
/// builder
///     .plugin(
///         tauri_plugin_http::Builder::new()
///             .user_agent("my-app/1.0")
///             .on_request(|ctx, headers| {
///                 if ctx.url().host_str() == Some("api.example.com") && !ctx.frontend_set(&ORIGIN) {
///                     headers.insert(ORIGIN, HeaderValue::from_static("https://desktop.example.com"));
///                 }
///             })
///             .build(),
///     )
/// # }
/// ```
#[derive(Default)]
pub struct Builder {
    user_agent: Option<String>,
    request_headers: Option<Vec<RequestHeaderRule>>,
    on_request: Option<request_headers::OnRequest>,
}

impl Builder {
    /// Creates a builder with the default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the `User-Agent` sent when the frontend does not set one, replacing
    /// [`Config::user_agent`].
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent.replace(user_agent.into());
        self
    }

    /// Sets the rules adding or removing headers on webview requests, replacing
    /// [`Config::request_headers`].
    pub fn request_headers(mut self, rules: impl IntoIterator<Item = RequestHeaderRule>) -> Self {
        self.request_headers.replace(rules.into_iter().collect());
        self
    }

    /// Sets a hook that can change the headers of every request sent from the webview.
    ///
    /// The hook runs once the request is allowed by the scope, after the plugin's default
    /// `User-Agent` and `Origin` and the [request header rules](Self::request_headers) were applied,
    /// so it sees and can change all of them. It runs again for every redirect the plugin follows,
    /// with the redirect target as [`RequestContext::url`] and the headers the frontend sent, so
    /// what it adds for one URL is not sent to another.
    ///
    /// The hook can set any header for any destination, including the ones the frontend is not
    /// allowed to send, such as `Origin`. Misrepresenting the client to third-party services may
    /// breach their terms of service.
    pub fn on_request<F>(mut self, on_request: F) -> Self
    where
        F: Fn(&RequestContext<'_>, &mut HeaderMap) + Send + Sync + 'static,
    {
        self.on_request.replace(Box::new(on_request));
        self
    }

    /// Validates the options, taking the ones that are not set on the builder from the `config`.
    fn into_request_headers(
        self,
        config: &Config,
    ) -> std::result::Result<request_headers::RequestHeaders, request_headers::InvalidRequestHeaders>
    {
        request_headers::RequestHeaders::new(
            self.user_agent.as_deref().or(config.user_agent.as_deref()),
            self.request_headers
                .as_deref()
                .unwrap_or(&config.request_headers),
            self.on_request,
        )
    }

    /// Builds the plugin.
    ///
    /// The setup fails if the user agent or a request header rule is invalid.
    pub fn build<R: Runtime>(self) -> TauriPlugin<R, Option<Config>> {
        PluginBuilder::<R, Option<Config>>::new("http")
            .setup(move |app, api| {
                let config = api.config().clone().unwrap_or_default();

                let request_headers = std::sync::Arc::new(self.into_request_headers(&config)?);

                #[cfg(feature = "cookies")]
                let cookies_jar = {
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
                    CookieJar::load(path.clone(), reader).unwrap_or_else(|_e| {
                        #[cfg(feature = "tracing")]
                        tracing::warn!(
                            "failed to load cookie store: {_e}, falling back to empty store"
                        );
                        CookieJar::new(path, Default::default())
                    })
                };

                let state = Http {
                    config,
                    request_headers,
                    #[cfg(feature = "cookies")]
                    cookies_jar: std::sync::Arc::new(cookies_jar),
                };

                app.manage(state);

                Ok(())
            })
            .on_event(|_app, _event| {
                #[cfg(feature = "cookies")]
                if let tauri::RunEvent::Exit = _event
                    && let Err(_e) = _app.http().cookies_jar.save()
                {
                    #[cfg(feature = "tracing")]
                    tracing::error!("failed to save cookie jar: {_e}");
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request_headers::tests::{apply, header};

    fn config(json: &str) -> Config {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn builder_options_take_precedence_over_the_config() {
        let config = config(
            r#"{
                "userAgent": "config/1.0",
                "requestHeaders": [
                    { "url": "https://api.example.com", "headers": { "X-Source": "config" } }
                ]
            }"#,
        );

        // nothing set on the builder: the config applies
        let request_headers = Builder::new().into_request_headers(&config).unwrap();
        let headers = apply(&request_headers, "https://api.example.com", &[]);
        assert_eq!(header(&headers, "user-agent"), Some("config/1.0"));
        assert_eq!(header(&headers, "x-source"), Some("config"));

        // the builder's rules replace the config's list instead of extending it
        let request_headers = Builder::new()
            .user_agent("builder/1.0")
            .request_headers([
                RequestHeaderRule::new("https://other.example.com").header("X-Source", "builder")
            ])
            .into_request_headers(&config)
            .unwrap();
        let headers = apply(&request_headers, "https://api.example.com", &[]);
        assert_eq!(header(&headers, "user-agent"), Some("builder/1.0"));
        assert_eq!(header(&headers, "x-source"), None);
        let headers = apply(&request_headers, "https://other.example.com", &[]);
        assert_eq!(header(&headers, "x-source"), Some("builder"));

        // an empty list on the builder disables the config's rules
        let request_headers = Builder::new()
            .request_headers([])
            .into_request_headers(&config)
            .unwrap();
        let headers = apply(&request_headers, "https://api.example.com", &[]);
        assert_eq!(header(&headers, "user-agent"), Some("config/1.0"));
        assert_eq!(header(&headers, "x-source"), None);
    }

    #[test]
    fn invalid_builder_options_are_rejected() {
        let config = Config::default();
        assert!(
            Builder::new()
                .user_agent("line\nbreak")
                .into_request_headers(&config)
                .is_err()
        );
        assert!(
            Builder::new()
                .request_headers([RequestHeaderRule::new("https://*").header("Cookie", "a=b")])
                .into_request_headers(&config)
                .is_err()
        );

        // an invalid config is not used when the builder replaces it
        let config: Config = serde_json::from_str(
            r#"{ "requestHeaders": [{ "url": "https://*", "headers": { "Host": "x" } }] }"#,
        )
        .unwrap();
        assert!(Builder::new().into_request_headers(&config).is_err());
        assert!(
            Builder::new()
                .request_headers([])
                .into_request_headers(&config)
                .is_ok()
        );
    }

    #[test]
    fn hook_is_kept() {
        let request_headers = Builder::new()
            .on_request(|_, headers| {
                headers.insert("x-hook", reqwest::header::HeaderValue::from_static("1"));
            })
            .into_request_headers(&Config::default())
            .unwrap();
        let headers = apply(&request_headers, "https://api.example.com", &[]);
        assert_eq!(header(&headers, "x-hook"), Some("1"));
    }
}
