// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/pfernie/reqwest_cookie_store/blob/2ec4afabcd55e24d3afe3f0626ee6dc97bed938d/src/lib.rs

use std::{
    fs::File,
    io::BufWriter,
    path::PathBuf,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use cookie_store::{CookieStore, RawCookie, RawCookieParseError};
use reqwest::header::HeaderValue;
use serde::{Deserialize, Serialize};

fn set_cookies(
    cookie_store: &mut CookieStore,
    cookie_headers: &mut dyn Iterator<Item = &HeaderValue>,
    url: &url::Url,
) {
    let cookies = cookie_headers.filter_map(|val| {
        std::str::from_utf8(val.as_bytes())
            .map_err(RawCookieParseError::from)
            .and_then(RawCookie::parse)
            .map(|c| c.into_owned())
            .ok()
    });
    cookie_store.store_response_cookies(cookies, url);
}

fn cookies(cookie_store: &CookieStore, url: &url::Url) -> Option<HeaderValue> {
    let s = cookie_store
        .get_request_values(url)
        .map(|(name, value)| format!("{}={}", name, value))
        .collect::<Vec<_>>()
        .join("; ");

    if s.is_empty() {
        return None;
    }

    HeaderValue::from_maybe_shared(bytes::Bytes::from(s)).ok()
}

/// A [`cookie_store::CookieStore`] wrapped internally by a [`std::sync::Mutex`], suitable for use in
/// async/concurrent contexts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CookieStoreMutex {
    pub path: PathBuf,
    store: Arc<Mutex<CookieStore>>,
}

impl CookieStoreMutex {
    /// Create a new [`CookieStoreMutex`] from an existing [`cookie_store::CookieStore`].
    pub fn new(path: PathBuf, cookie_store: CookieStore) -> CookieStoreMutex {
        CookieStoreMutex {
            path,
            store: Arc::new(Mutex::new(cookie_store)),
        }
    }

    /// Lock and get a handle to the contained [`cookie_store::CookieStore`].
    pub fn lock(
        &self,
    ) -> Result<MutexGuard<'_, CookieStore>, PoisonError<MutexGuard<'_, CookieStore>>> {
        self.store.lock()
    }

    pub fn load<R: std::io::BufRead>(
        path: PathBuf,
        reader: R,
    ) -> cookie_store::Result<CookieStoreMutex> {
        cookie_store::serde::load(reader, |c| serde_json::from_str(c))
            .map(|store| CookieStoreMutex::new(path, store))
    }

    pub fn save(&self) -> cookie_store::Result<()> {
        let file = File::create(&self.path)?;
        let mut writer = BufWriter::new(file);
        let store = self.lock().expect("poisoned cookie jar mutex");
        cookie_store::serde::save(&store, &mut writer, serde_json::to_string)
    }
}

impl reqwest::cookie::CookieStore for CookieStoreMutex {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        let mut store = self.store.lock().unwrap();
        set_cookies(&mut store, cookie_headers, url);

        // try to persist cookies immediately asynchronously
        let cookies_jar = self.clone();
        tauri::async_runtime::spawn(async move {
            if let Err(_e) = cookies_jar.save() {
                #[cfg(feature = "tracing")]
                tracing::error!("failed to save cookie jar: {_e}");
            }
        });
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let store = self.store.lock().unwrap();
        cookies(&store, url)
    }
}
