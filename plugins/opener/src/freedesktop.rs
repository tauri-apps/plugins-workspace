// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Freedesktop integration for opening files, folders and URLs.
//!
//! `open` resolves launch helpers through `PATH`. AppImages prepend their own
//! `usr/bin`, so a bundled `xdg-open` can shadow the host binary and swallow
//! the request without opening anything. The file manager and the XDG desktop
//! portal accept the same request over D-Bus, which always reaches the host
//! desktop.

use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use zbus::zvariant::{OwnedObjectPath, Value};

/// What a path passed to `open` should be routed to.
#[derive(Debug)]
enum Target {
    Folder(PathBuf),
    File(PathBuf),
    Uri(url::Url),
}

/// Picks the desktop API for `path`, or `None` when it is neither a local path
/// nor a parseable URL.
fn classify(path: &OsStr) -> Option<Target> {
    let path = Path::new(path);
    if path.is_dir() {
        return Some(Target::Folder(path.to_path_buf()));
    }
    if path.is_file() {
        return Some(Target::File(path.to_path_buf()));
    }

    url::Url::parse(&path.to_string_lossy())
        .ok()
        .filter(|url| url.scheme() != "file")
        .map(Target::Uri)
}

/// Opens `path` through the desktop's D-Bus services.
///
/// Returns `None` when the target cannot be classified. Errors mean the
/// desktop did not provide the service, so callers can fall back to `open`.
pub(crate) fn try_open(path: &OsStr) -> Option<crate::Result<()>> {
    match classify(path) {
        Some(Target::Folder(path)) => Some(show_folder(&path)),
        Some(Target::File(path)) => Some(open_file(&path)),
        Some(Target::Uri(uri)) => Some(open_uri(&uri)),
        None => None,
    }
}

fn show_folder(path: &Path) -> crate::Result<()> {
    let uri = file_url(path)?;
    on_blocking_thread(|connection| {
        let proxy = FileManager1ProxyBlocking::new(connection)?;
        proxy
            .ShowFolders(vec![uri.as_str()], "")
            .map_err(Into::into)
    })
}

fn open_file(path: &Path) -> crate::Result<()> {
    let uri = file_url(path)?;
    on_blocking_thread(|connection| {
        let proxy = OpenUriProxyBlocking::new(connection)?;
        proxy
            .OpenFile("", uri.as_str(), HashMap::new())
            .map(|_| ())
            .map_err(Into::into)
    })
}

fn open_uri(uri: &url::Url) -> crate::Result<()> {
    on_blocking_thread(|connection| {
        let proxy = OpenUriProxyBlocking::new(connection)?;
        proxy
            .OpenURI("", uri.as_str(), HashMap::new())
            .map(|_| ())
            .map_err(Into::into)
    })
}

fn file_url(path: &Path) -> crate::Result<url::Url> {
    url::Url::from_file_path(path).map_err(|_| crate::Error::FailedToConvertPathToFileUrl)
}

/// Runs `task` with a blocking D-Bus connection on a dedicated thread.
///
/// `zbus::blocking` starts its own tokio runtime when zbus is built with the
/// `tokio` feature, which panics with "Cannot start a runtime from within a
/// runtime" when called from an async command. A dedicated thread avoids that
/// and keeps the async runtime free during the D-Bus round trip.
fn on_blocking_thread(
    task: impl FnOnce(&zbus::blocking::Connection) -> crate::Result<()> + Send,
) -> crate::Result<()> {
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let connection = zbus::blocking::Connection::session()?;
                task(&connection)
            })
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}

#[zbus::proxy(
    interface = "org.freedesktop.FileManager1",
    default_service = "org.freedesktop.FileManager1",
    default_path = "/org/freedesktop/FileManager1"
)]
trait FileManager1 {
    #[allow(non_snake_case)]
    fn ShowFolders(&self, folders: Vec<&str>, startup_id: &str) -> zbus::Result<()>;
}

#[zbus::proxy(
    interface = "org.freedesktop.portal.OpenURI",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait OpenUri {
    #[allow(non_snake_case)]
    fn OpenURI(
        &self,
        parent_window: &str,
        uri: &str,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;

    #[allow(non_snake_case)]
    fn OpenFile(
        &self,
        parent_window: &str,
        uri: &str,
        options: HashMap<&str, Value<'_>>,
    ) -> zbus::Result<OwnedObjectPath>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_paths_and_urls() {
        assert!(matches!(
            classify(OsStr::new("Cargo.toml")),
            Some(Target::File(_))
        ));
        assert!(matches!(
            classify(OsStr::new("src")),
            Some(Target::Folder(_))
        ));
        assert!(matches!(
            classify(OsStr::new("https://example.com")),
            Some(Target::Uri(_))
        ));
        assert!(classify(OsStr::new("does-not-exist")).is_none());
    }
}
