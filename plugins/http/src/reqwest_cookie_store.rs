// taken from https://github.com/pfernie/reqwest_cookie_store/blob/2ec4afabcd55e24d3afe3f0626ee6dc97bed938d/src/lib.rs

use std::sync::{Mutex, MutexGuard, PoisonError};

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
#[derive(Debug, Serialize, Deserialize)]
pub struct CookieStoreMutex(Mutex<CookieStore>);

impl Default for CookieStoreMutex {
    /// Create a new, empty [`CookieStoreMutex`]
    fn default() -> Self {
        CookieStoreMutex::new(CookieStore::default())
    }
}

impl CookieStoreMutex {
    /// Create a new [`CookieStoreMutex`] from an existing [`cookie_store::CookieStore`].
    pub const fn new(cookie_store: CookieStore) -> CookieStoreMutex {
        CookieStoreMutex(Mutex::new(cookie_store))
    }

    /// Lock and get a handle to the contained [`cookie_store::CookieStore`].
    pub fn lock(
        &self,
    ) -> Result<MutexGuard<'_, CookieStore>, PoisonError<MutexGuard<'_, CookieStore>>> {
        self.0.lock()
    }

    pub fn load<R: std::io::BufRead>(reader: R) -> cookie_store::Result<CookieStoreMutex> {
        cookie_store::serde::load(reader, |c| serde_json::from_str(c)).map(CookieStoreMutex::new)
    }

    pub fn save<W: std::io::Write>(&self, writer: &mut W) -> cookie_store::Result<()> {
        let store = self.lock().expect("poisoned cookie jar mutex");
        cookie_store::serde::save(&store, writer, |c| serde_json::to_string(c))
    }
}

impl reqwest::cookie::CookieStore for CookieStoreMutex {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        let mut store = self.0.lock().unwrap();
        set_cookies(&mut store, cookie_headers, url);
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let store = self.0.lock().unwrap();
        cookies(&store, url)
    }
}
