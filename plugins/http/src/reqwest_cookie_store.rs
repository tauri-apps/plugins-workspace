// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// taken from https://github.com/pfernie/reqwest_cookie_store/blob/2ec4afabcd55e24d3afe3f0626ee6dc97bed938d/src/lib.rs

use std::{
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};

use cookie_store::{CookieStore, RawCookie, RawCookieParseError};
use reqwest::header::HeaderValue;

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
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("; ");

    if s.is_empty() {
        return None;
    }

    HeaderValue::from_maybe_shared(bytes::Bytes::from(s)).ok()
}

/// The file the cookie jar is persisted to.
///
/// Every snapshot of the jar gets an increasing generation, and a snapshot is only written if no
/// newer one has been written yet, so concurrent saves cannot replace the file with stale cookies.
struct CookieFile {
    path: PathBuf,
    /// The generation of the latest snapshot.
    latest: AtomicU64,
    /// The generation of the snapshot on disk.
    written: Mutex<u64>,
}

impl CookieFile {
    /// Writes the snapshot of the given generation.
    ///
    /// With `skip_if_superseded`, the write is also skipped when a newer snapshot exists, because
    /// whoever took that snapshot writes it.
    fn write(
        &self,
        generation: u64,
        contents: &str,
        skip_if_superseded: bool,
    ) -> std::io::Result<()> {
        let mut written = self.written.lock().unwrap();
        if *written >= generation
            || (skip_if_superseded && self.latest.load(Ordering::SeqCst) > generation)
        {
            return Ok(());
        }
        std::fs::write(&self.path, contents)?;
        *written = generation;
        Ok(())
    }
}

/// The cookie jar shared by every request the plugin sends.
///
/// Get it with [`Http::cookie_jar`](crate::Http::cookie_jar). It implements
/// [`reqwest::cookie::CookieStore`], so it can be given to your own [`reqwest`] clients with
/// [`ClientBuilder::cookie_provider`](reqwest::ClientBuilder::cookie_provider) to share the
/// session with the webview:
///
/// ```rust,no_run
/// use tauri_plugin_http::HttpExt;
///
/// # fn f<R: tauri::Runtime>(app: &tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
/// let client = tauri_plugin_http::reqwest::Client::builder()
///     .cookie_provider(app.http().cookie_jar())
///     .build()?;
/// # Ok(())
/// # }
/// ```
///
/// The jar holds the session cookies of every request, so any Rust code with access to it can read
/// and use them.
///
/// Persistent cookies are written to a `.cookies` file in the application cache directory. The file
/// is updated in the background whenever the jar changes, and once more, synchronously, when the
/// application exits. Use [`CookieJar::save`] to write it immediately.
pub struct CookieJar {
    store: Mutex<CookieStore>,
    file: Arc<CookieFile>,
}

impl std::fmt::Debug for CookieJar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // the cookies are session secrets, so they are left out
        f.debug_struct("CookieJar")
            .field("path", &self.file.path)
            .finish_non_exhaustive()
    }
}

impl CookieJar {
    pub(crate) fn new(path: PathBuf, cookie_store: CookieStore) -> CookieJar {
        CookieJar {
            store: Mutex::new(cookie_store),
            file: Arc::new(CookieFile {
                path,
                latest: AtomicU64::new(0),
                written: Mutex::new(0),
            }),
        }
    }

    pub(crate) fn load<R: std::io::BufRead>(
        path: PathBuf,
        reader: R,
    ) -> cookie_store::Result<CookieJar> {
        cookie_store::serde::load(reader, |c| serde_json::from_str(c))
            .map(|store| CookieJar::new(path, store))
    }

    /// Removes every cookie from the jar.
    ///
    /// The change is persisted in the background, like any other change to the jar.
    pub fn clear(&self) {
        self.store.lock().unwrap().clear();
        self.request_save();
    }

    /// Removes the cookie with the given `domain`, `path` and `name`, returning whether it existed.
    ///
    /// The change is persisted in the background, like any other change to the jar.
    pub fn remove(&self, domain: &str, path: &str, name: &str) -> bool {
        let removed = self
            .store
            .lock()
            .unwrap()
            .remove(domain, path, name)
            .is_some();
        if removed {
            self.request_save();
        }
        removed
    }

    /// Writes the persistent cookies to disk now, blocking until the file is written.
    pub fn save(&self) -> crate::Result<()> {
        let (generation, contents) = self.snapshot()?;
        self.file.write(generation, &contents, false)?;
        Ok(())
    }

    /// Serializes the persistent cookies and gives the snapshot a new generation.
    fn snapshot(&self) -> serde_json::Result<(u64, String)> {
        // the generation is taken under the store lock so it follows the order of the changes
        let store = self.store.lock().unwrap();
        let cookies = store
            .iter_unexpired()
            .filter(|cookie| cookie.is_persistent())
            .collect::<Vec<_>>();
        let contents = serde_json::to_string(&cookies)?;
        let generation = self.file.latest.fetch_add(1, Ordering::SeqCst) + 1;
        Ok((generation, contents))
    }

    /// Writes the persistent cookies to disk in the background.
    fn request_save(&self) {
        let (generation, contents) = match self.snapshot() {
            Ok(snapshot) => snapshot,
            Err(_e) => {
                #[cfg(feature = "tracing")]
                tracing::error!("failed to save cookie jar: {_e}");
                return;
            }
        };
        let file = self.file.clone();
        tauri::async_runtime::spawn_blocking(move || {
            if let Err(_e) = file.write(generation, &contents, true) {
                #[cfg(feature = "tracing")]
                tracing::error!("failed to save cookie jar: {_e}");
            }
        });
    }
}

impl reqwest::cookie::CookieStore for CookieJar {
    fn set_cookies(&self, cookie_headers: &mut dyn Iterator<Item = &HeaderValue>, url: &url::Url) {
        set_cookies(&mut self.store.lock().unwrap(), cookie_headers, url);

        // try to persist cookies immediately asynchronously
        self.request_save();
    }

    fn cookies(&self, url: &url::Url) -> Option<HeaderValue> {
        let store = self.store.lock().unwrap();
        cookies(&store, url)
    }
}

#[cfg(test)]
mod tests {
    use std::{
        io::{BufRead, BufReader, Write},
        net::TcpListener,
        sync::Arc,
    };

    use super::CookieJar;

    /// Test server where `/login` sets a persistent `session` cookie, and every other path answers
    /// with the `Cookie` header it received.
    fn spawn_server() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();

        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let mut stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                let mut cookie = String::new();
                loop {
                    let mut header = String::new();
                    reader.read_line(&mut header).unwrap();
                    if matches!(header.as_str(), "" | "\r\n" | "\n") {
                        break;
                    }
                    if let Some(value) = header.strip_prefix("cookie: ") {
                        cookie = value.trim().to_string();
                    }
                }

                let path = request_line.split(' ').nth(1).unwrap_or("/");
                let response = if path == "/login" {
                    "HTTP/1.1 200 OK\r\nSet-Cookie: session=abc; Max-Age=3600\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_string()
                } else {
                    format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{cookie}",
                        cookie.len()
                    )
                };
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.flush();
            }
        });

        port
    }

    fn get(jar: &Arc<CookieJar>, url: &str) -> String {
        let client = reqwest::Client::builder()
            .cookie_provider(jar.clone())
            .build()
            .unwrap();
        tauri::async_runtime::block_on(async move {
            client.get(url).send().await.unwrap().text().await.unwrap()
        })
    }

    fn temp_jar(name: &str) -> (std::path::PathBuf, Arc<CookieJar>) {
        let path = std::env::temp_dir().join(format!(
            "tauri-plugin-http-{name}-{}.cookies",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let jar = Arc::new(CookieJar::new(path.clone(), Default::default()));
        (path, jar)
    }

    #[test]
    fn clients_sharing_the_jar_share_the_session() {
        let port = spawn_server();
        let (path, jar) = temp_jar("shared");

        // the plugin's client logs in, then a separate client sends the session cookie
        get(&jar, &format!("http://localhost:{port}/login"));
        assert_eq!(
            get(&jar, &format!("http://localhost:{port}/me")),
            "session=abc"
        );

        jar.save().unwrap();
        let saved = std::fs::read_to_string(&path).unwrap();
        assert!(saved.contains("abc"));

        // removing a cookie that does not exist is a no-op
        assert!(!jar.remove("localhost", "/", "missing"));
        assert!(jar.remove("localhost", "/", "session"));
        assert_eq!(get(&jar, &format!("http://localhost:{port}/me")), "");

        get(&jar, &format!("http://localhost:{port}/login"));
        jar.clear();
        assert_eq!(get(&jar, &format!("http://localhost:{port}/me")), "");

        jar.save().unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "[]");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn saved_cookies_are_loaded_back() {
        let port = spawn_server();
        let (path, jar) = temp_jar("reload");

        get(&jar, &format!("http://localhost:{port}/login"));
        jar.save().unwrap();

        let file = std::fs::File::open(&path).unwrap();
        let reloaded = Arc::new(CookieJar::load(path.clone(), BufReader::new(file)).unwrap());
        assert_eq!(
            get(&reloaded, &format!("http://localhost:{port}/me")),
            "session=abc"
        );

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn debug_output_leaves_the_cookies_out() {
        let port = spawn_server();
        let (path, jar) = temp_jar("debug");

        get(&jar, &format!("http://localhost:{port}/login"));
        let debug = format!("{jar:?}");
        assert!(debug.contains("CookieJar"), "{debug}");
        assert!(!debug.contains("abc"), "{debug}");

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn stale_snapshots_are_not_written() {
        let (path, jar) = temp_jar("stale");

        let (old_generation, old) = jar.snapshot().unwrap();
        jar.save().unwrap();
        std::fs::write(&path, "newer").unwrap();

        // an older background save finishing late must not replace the newer file
        jar.file.write(old_generation, &old, true).unwrap();
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "newer");

        let _ = std::fs::remove_file(&path);
    }
}
