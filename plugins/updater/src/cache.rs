// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt::Debug, time::Duration};

use http_cache_reqwest::{
    Cache, CacheOptions, HttpCache, HttpCacheOptions, MokaCache, MokaManager,
};
use serde::{Deserialize, Serialize};

/// Represents a caching behavior for update checks.
///
/// See the individual variants for more details on which to use.
#[derive(Debug, Default, Clone, Copy, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CacheMode {
    /// Default caching mode is useful for UI display purposes and background checks.
    /// Serves from cache so long as HTTP headers allow it.
    #[default]
    Default,

    /// Completely skips caching and makes a new server request every time.
    /// Doesn't read from cache, doesn't update cache.
    ///
    /// Note: a future request that does use caching may return stale results.
    /// Use `CheckNow` when the cache should store the latest response.
    Bypass,

    /// Recommended for a "Check for updates now" feature where the user actively wants to refresh.
    /// Always makes a server request, but can use conditional requests for example using the etag.
    /// Updates the cache with the response.
    ///
    /// Note: this sets a `Cache-Control: no-cache` request header, which requests that the server
    /// also skips any CDN caches and requests updates from the origin server.
    /// This should NOT be used for automated background checking.
    CheckNow,
}

impl From<CacheMode> for http_cache_reqwest::CacheMode {
    fn from(value: CacheMode) -> Self {
        match value {
            CacheMode::Default => http_cache_reqwest::CacheMode::Default,
            CacheMode::Bypass => http_cache_reqwest::CacheMode::NoStore,
            CacheMode::CheckNow => http_cache_reqwest::CacheMode::NoCache,
        }
    }
}

/// Cache configuration for the updater.
///
/// Generally should be set through your `tauri.conf.json` settings.
#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CacheConfig {
    /// Whether to fully disable HTTP caching on update requests.
    /// This also ignores any per-request cache options.
    #[serde(default)]
    pub disabled: bool,
    /// Changes which `CacheMode` is used when no per-request override is given.
    /// Unlike the `disabled` config this allows selectively caching requests.
    ///
    /// false = `CacheMode::Default`
    /// true = `CacheMode::Bypass`
    #[serde(default, alias = "disabled-by-default")]
    pub disabled_by_default: bool,
    /// Sets an upper limit in minutes to how long a response may be cached.
    /// It may be shorter depending on the `cache-control` or `expires` server headers.
    /// Responses without those headers are cached for 1 hour, or this limit if lower.
    /// Defaults to 12 hours.
    #[serde(default, alias = "max-ttl-mins")]
    pub max_ttl_mins: Option<u32>,
}

/// TTL for responses where the server doesn't set `max-age` or `expires` headers.
///
/// Note: not configurable on purpose. We shouldn't expose this until the upstream
/// `default_ttl` behavior is settled, see https://github.com/06chaynes/http-cache/issues/175
const DEFAULT_TTL: Duration = Duration::from_secs(60 * 60);

/// The cache capacity per created MokaManager instance.
/// Generally this would be allocated once for application-wide use.
///
/// Assumes 1-3 endpoints might typically be used.
/// Assumes redirects don't take up a slot.
/// Leaves some overhead for changing inputs.
const CACHE_CAPACITY: u64 = 6;

/// A cache manager for the updater.
///
/// Uses an internal `Arc` so cloning means we use a shared cache.
/// Use `from_config` to create a new instance with separate cache storage.
#[derive(Debug, Default, Clone)]
pub struct CacheManager {
    // None here implies cache is fully disabled.
    http_cache: Option<MokaManager>,

    // Supplied as-is to `HttpCacheOptions`.
    // None means no limit and it will cache however long the server suggests.
    max_ttl: Option<Duration>,

    // Mode to use when no override is set.
    default_mode: CacheMode,
}

impl CacheManager {
    /// Creates a new [`CacheManager`] from config.
    ///
    /// This will also create a new cache storage (if caching is enabled).
    /// For a shared cache storage, you should clone an existing [`CacheManager`].
    pub fn from_config(config: CacheConfig) -> Self {
        Self {
            http_cache: if !config.disabled {
                Some(MokaManager::new(MokaCache::new(CACHE_CAPACITY)))
            } else {
                None
            },
            max_ttl: config
                .max_ttl_mins
                .or(Some(12 * 60))
                .map(|mins| Duration::from_secs(mins as u64 * 60)),
            default_mode: if !config.disabled_by_default {
                CacheMode::Default
            } else {
                CacheMode::Bypass
            },
        }
    }

    /// Shorthand to check if caching is enabled.
    pub(crate) fn enabled(&self) -> bool {
        self.http_cache.is_some()
    }

    /// Provides a caching middleware for use with `reqwest-middleware` if caching is enabled.
    /// `None` when caching is disabled.
    pub(crate) fn maybe_middleware(
        &self,
        mode_override: Option<CacheMode>,
    ) -> Option<Cache<MokaManager>> {
        let Some(http_cache) = &self.http_cache else {
            return None;
        };

        Some(Cache(HttpCache {
            mode: mode_override.unwrap_or(self.default_mode).into(),
            manager: http_cache.clone(),
            options: HttpCacheOptions {
                default_ttl: Some(DEFAULT_TTL),
                max_ttl: self.max_ttl,
                cache_options: Some(CacheOptions {
                    // This is a private cache for one client, hence shared should be false.
                    shared: false,
                    ..Default::default()
                }),
                ..Default::default()
            },
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use http_cache_reqwest::CacheMode as HttpCacheMode;

    fn parse_config(json: &str) -> CacheConfig {
        serde_json::from_str(json).expect("invalid cache config")
    }

    fn enabled() -> CacheManager {
        CacheManager::from_config(CacheConfig::default())
    }

    /// The mode the middleware applies to a request, panicking if caching is disabled.
    fn applied_mode(cache: &CacheManager, mode_override: Option<CacheMode>) -> HttpCacheMode {
        cache
            .maybe_middleware(mode_override)
            .expect("cache is disabled")
            .0
            .mode
    }

    #[test]
    fn config_defaults() {
        let config = parse_config("{}");
        assert!(!config.disabled);
        assert!(!config.disabled_by_default);
        assert_eq!(config.max_ttl_mins, None);
    }

    #[test]
    fn config_accepts_camel_and_kebab_case() {
        for json in [
            r#"{ "disabled": true, "disabledByDefault": true, "maxTtlMins": 5 }"#,
            r#"{ "disabled": true, "disabled-by-default": true, "max-ttl-mins": 5 }"#,
        ] {
            let config = parse_config(json);
            assert!(config.disabled, "{json}");
            assert!(config.disabled_by_default, "{json}");
            assert_eq!(config.max_ttl_mins, Some(5), "{json}");
        }
    }

    #[test]
    fn updater_config_reads_cache_section() {
        let config: crate::Config =
            serde_json::from_str(r#"{ "pubkey": "", "cache": { "maxTtlMins": 5 } }"#).unwrap();
        assert_eq!(config.cache.unwrap().max_ttl_mins, Some(5));

        let config: crate::Config = serde_json::from_str(r#"{ "pubkey": "" }"#).unwrap();
        assert!(config.cache.is_none());
    }

    #[test]
    fn cache_mode_uses_js_names() {
        for (mode, name) in [
            (CacheMode::Default, "default"),
            (CacheMode::Bypass, "bypass"),
            (CacheMode::CheckNow, "checkNow"),
        ] {
            assert_eq!(serde_json::to_string(&mode).unwrap(), format!("\"{name}\""));
            assert_eq!(
                std::mem::discriminant(
                    &serde_json::from_str::<CacheMode>(&format!("\"{name}\"")).unwrap()
                ),
                std::mem::discriminant(&mode)
            );
        }
        assert!(serde_json::from_str::<CacheMode>("\"disabled\"").is_err());
    }

    #[test]
    fn default_config_enables_private_cache_with_1h_default_and_12h_max_ttl() {
        let cache = enabled();
        let middleware = cache.maybe_middleware(None).expect("cache is disabled");
        assert_eq!(middleware.0.mode, HttpCacheMode::Default);
        assert_eq!(
            middleware.0.options.default_ttl,
            Some(Duration::from_secs(60 * 60))
        );
        assert_eq!(
            middleware.0.options.max_ttl,
            Some(Duration::from_secs(12 * 60 * 60))
        );
        assert!(!middleware.0.options.cache_options.unwrap().shared);
    }

    #[test]
    fn max_ttl_is_in_minutes() {
        let cache = CacheManager::from_config(parse_config(r#"{ "maxTtlMins": 5 }"#));
        assert_eq!(cache.max_ttl, Some(Duration::from_secs(5 * 60)));
    }

    #[test]
    fn mode_override_takes_precedence() {
        let cache = enabled();
        assert_eq!(applied_mode(&cache, None), HttpCacheMode::Default);
        assert_eq!(
            applied_mode(&cache, Some(CacheMode::CheckNow)),
            HttpCacheMode::NoCache
        );
    }

    #[test]
    fn disabled_by_default_still_allows_override() {
        let cache = CacheManager::from_config(parse_config(r#"{ "disabledByDefault": true }"#));
        assert_eq!(applied_mode(&cache, None), HttpCacheMode::NoStore);
        assert_eq!(
            applied_mode(&cache, Some(CacheMode::Default)),
            HttpCacheMode::Default
        );
    }

    #[test]
    fn disabled_ignores_override() {
        let cache = CacheManager::from_config(parse_config(r#"{ "disabled": true }"#));
        for mode in [
            None,
            Some(CacheMode::Default),
            Some(CacheMode::Bypass),
            Some(CacheMode::CheckNow),
        ] {
            assert!(cache.maybe_middleware(mode).is_none(), "{mode:?}");
        }
    }

    /// These run the middleware against a local server and count the requests that reach it, to
    /// check how our configuration of `http-cache` behaves rather than the library itself.
    mod http {
        use super::*;

        use mockito::{Matcher, Server};

        const ETAG: &str = "\"v1\"";

        /// Sends a GET the way `Updater::check` does, returning the status and body.
        async fn get(
            cache: &CacheManager,
            mode_override: Option<CacheMode>,
            url: &str,
            headers: &[(&str, &str)],
        ) -> (u16, String) {
            #[cfg(feature = "rustls-tls")]
            if rustls::crypto::CryptoProvider::get_default().is_none() {
                let _ = rustls::crypto::ring::default_provider().install_default();
            }

            let client = reqwest::ClientBuilder::new().no_proxy().build().unwrap();
            let mut client = reqwest_middleware::ClientBuilder::new(client);
            if let Some(cache) = cache.maybe_middleware(mode_override) {
                client = client.with(cache);
            }

            let mut request = client.build().get(url);
            for (name, value) in headers {
                request = request.header(*name, *value);
            }
            let response = request.send().await.expect("request failed");
            let status = response.status().as_u16();
            (status, response.text().await.unwrap())
        }

        /// Asserts how many of `count` GETs with `mode_override` reach a server responding with
        /// `status` and `headers`.
        async fn assert_hits(
            cache: &CacheManager,
            mode_override: Option<CacheMode>,
            status: usize,
            headers: &[(&str, &str)],
            count: usize,
            expected_hits: usize,
        ) {
            let mut server = Server::new_async().await;
            let mut mock = server.mock("GET", "/").with_status(status);
            for (name, value) in headers {
                mock = mock.with_header(*name, value);
            }
            let mock = mock.expect(expected_hits).create_async().await;

            for _ in 0..count {
                get(cache, mode_override, &server.url(), &[]).await;
            }
            mock.assert_async().await;
        }

        #[tokio::test]
        async fn serves_fresh_response_from_cache() {
            let cache = enabled();
            assert_hits(&cache, None, 200, &[("cache-control", "max-age=60")], 3, 1).await;
        }

        #[tokio::test]
        async fn caches_private_responses() {
            let cache = enabled();
            let headers = [("cache-control", "private, max-age=60")];
            assert_hits(&cache, None, 200, &headers, 3, 1).await;
        }

        #[tokio::test]
        async fn caches_no_update_response() {
            let cache = enabled();
            assert_hits(&cache, None, 204, &[("cache-control", "max-age=60")], 3, 1).await;
        }

        // `DEFAULT_TTL` applies to responses without a `max-age` or `expires`, rather than them
        // expiring immediately as they would under plain HTTP caching.
        #[tokio::test]
        async fn caches_response_without_caching_headers_for_default_ttl() {
            let cache = enabled();
            assert_hits(&cache, None, 200, &[], 3, 1).await;
            assert_hits(&cache, None, 204, &[], 3, 1).await;
        }

        #[tokio::test]
        async fn does_not_serve_expired_responses_from_cache() {
            let cache = enabled();
            let headers = [("expires", "Thu, 01 Jan 1970 00:00:00 GMT")];
            assert_hits(&cache, None, 200, &headers, 3, 3).await;
        }

        #[tokio::test]
        async fn does_not_store_no_store_responses() {
            let cache = enabled();
            assert_hits(&cache, None, 200, &[("cache-control", "no-store")], 3, 3).await;
        }

        #[tokio::test]
        async fn revalidates_no_cache_responses() {
            let cache = enabled();
            let mut server = Server::new_async().await;
            let full = server
                .mock("GET", "/")
                .match_header("if-none-match", Matcher::Missing)
                .with_header("cache-control", "no-cache")
                .with_header("etag", ETAG)
                .with_body("v1")
                .expect(1)
                .create_async()
                .await;
            // a 304 has to repeat the ETag, or the stored response is not updated from it
            let revalidated = server
                .mock("GET", "/")
                .match_header("if-none-match", ETAG)
                .with_status(304)
                .with_header("etag", ETAG)
                .expect(2)
                .create_async()
                .await;

            for _ in 0..3 {
                assert_eq!(
                    get(&cache, None, &server.url(), &[]).await,
                    (200, "v1".into())
                );
            }
            full.assert_async().await;
            revalidated.assert_async().await;
        }

        #[tokio::test]
        async fn max_ttl_caps_server_max_age_and_default_ttl() {
            for cache_control in [Some("max-age=3600"), None] {
                let cache = CacheManager {
                    max_ttl: Some(Duration::from_secs(1)),
                    ..enabled()
                };
                let mut server = Server::new_async().await;
                let mut mock = server.mock("GET", "/");
                if let Some(cache_control) = cache_control {
                    mock = mock.with_header("cache-control", cache_control);
                }
                let mock = mock.expect(2).create_async().await;

                get(&cache, None, &server.url(), &[]).await;
                get(&cache, None, &server.url(), &[]).await;
                tokio::time::sleep(Duration::from_millis(1500)).await;
                get(&cache, None, &server.url(), &[]).await;
                mock.assert_async().await;
            }
        }

        #[tokio::test]
        async fn clones_share_the_cache() {
            let cache = enabled();
            let mut server = Server::new_async().await;
            let mock = server
                .mock("GET", "/")
                .with_header("cache-control", "max-age=60")
                .expect(1)
                .create_async()
                .await;

            get(&cache, None, &server.url(), &[]).await;
            get(&cache.clone(), None, &server.url(), &[]).await;
            mock.assert_async().await;
        }

        #[tokio::test]
        async fn disabled_config_never_caches() {
            let cache = CacheManager::from_config(parse_config(r#"{ "disabled": true }"#));
            let headers = [("cache-control", "max-age=60")];
            assert_hits(&cache, None, 200, &headers, 3, 3).await;
            assert_hits(&cache, Some(CacheMode::Default), 200, &headers, 3, 3).await;
        }

        #[tokio::test]
        async fn disabled_by_default_allows_opting_in() {
            let cache = CacheManager::from_config(parse_config(r#"{ "disabledByDefault": true }"#));
            let mut server = Server::new_async().await;
            let mock = server
                .mock("GET", "/")
                .with_header("cache-control", "max-age=60")
                .expect(3)
                .create_async()
                .await;

            get(&cache, None, &server.url(), &[]).await;
            get(&cache, None, &server.url(), &[]).await;
            get(&cache, Some(CacheMode::Default), &server.url(), &[]).await;
            get(&cache, Some(CacheMode::Default), &server.url(), &[]).await;
            mock.assert_async().await;
        }

        #[tokio::test]
        async fn bypass_mode_neither_reads_nor_writes() {
            let cache = enabled();
            let mut server = Server::new_async().await;
            let mock = server
                .mock("GET", "/")
                .with_header("cache-control", "max-age=60")
                .expect(3)
                .create_async()
                .await;

            // not stored
            get(&cache, Some(CacheMode::Bypass), &server.url(), &[]).await;
            get(&cache, None, &server.url(), &[]).await;
            // not read
            get(&cache, Some(CacheMode::Bypass), &server.url(), &[]).await;
            get(&cache, None, &server.url(), &[]).await;
            mock.assert_async().await;
        }

        // `CheckNow` always reaches the server, but revalidates with a conditional request.
        #[tokio::test]
        async fn check_now_revalidates_and_stores() {
            let cache = enabled();
            let mut server = Server::new_async().await;
            let v1 = server
                .mock("GET", "/")
                .match_header("if-none-match", Matcher::Missing)
                .with_header("cache-control", "max-age=60")
                .with_header("etag", ETAG)
                .with_body("v1")
                .expect(1)
                .create_async()
                .await;
            get(&cache, None, &server.url(), &[]).await;
            v1.assert_async().await;

            let not_modified = server
                .mock("GET", "/")
                .match_header("if-none-match", ETAG)
                .match_header("cache-control", "no-cache")
                .with_status(304)
                .with_header("etag", ETAG)
                .expect(1)
                .create_async()
                .await;
            assert_eq!(
                get(&cache, Some(CacheMode::CheckNow), &server.url(), &[]).await,
                (200, "v1".into())
            );
            not_modified.assert_async().await;
            not_modified.remove_async().await;

            let v2 = server
                .mock("GET", "/")
                .match_header("if-none-match", ETAG)
                .match_header("cache-control", "no-cache")
                .with_header("cache-control", "max-age=60")
                .with_body("v2")
                .expect(1)
                .create_async()
                .await;
            assert_eq!(
                get(&cache, Some(CacheMode::CheckNow), &server.url(), &[]).await,
                (200, "v2".into())
            );
            assert_eq!(
                get(&cache, None, &server.url(), &[]).await,
                (200, "v2".into())
            );
            v2.assert_async().await;
        }

        // The cache key is the method and URL, request headers only count when listed in `Vary`.
        #[tokio::test]
        async fn request_headers_only_count_when_listed_in_vary() {
            for (vary, expected_hits) in [(None, 1), (Some("x-channel"), 2)] {
                let cache = enabled();
                let mut server = Server::new_async().await;
                let mut mock = server
                    .mock("GET", "/")
                    .with_header("cache-control", "max-age=60");
                if let Some(vary) = vary {
                    mock = mock.with_header("vary", vary);
                }
                let mock = mock.expect(expected_hits).create_async().await;

                get(&cache, None, &server.url(), &[("x-channel", "stable")]).await;
                get(&cache, None, &server.url(), &[("x-channel", "beta")]).await;
                mock.assert_async().await;
            }
        }
    }
}
