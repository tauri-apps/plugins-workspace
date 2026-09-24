// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt::Debug, time::Duration};

use http_cache_reqwest::{
    Cache, CacheOptions, HttpCache, HttpCacheOptions, MokaCache, MokaManager,
};
use serde::{Deserialize, Serialize};

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

#[derive(Debug, Default, Clone)]
pub struct CacheManager {
    // None here implies cache is fully disabled.
    // Uses an internal `Arc` so cloning means we use a shared cache.
    http_cache: Option<MokaManager>,

    // Supplied as-is to `HttpCacheOptions`.
    // None means no limit and it will cache however long the server suggests.
    max_ttl: Option<Duration>,

    // Mode to use when no override is set.
    default_mode: CacheMode,
}

impl CacheManager {
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

    pub(crate) fn enabled(&self) -> bool {
        self.http_cache.is_some()
    }

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

