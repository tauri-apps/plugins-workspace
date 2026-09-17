// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to shell.

use std::{ffi::OsStr, path::Path};

pub(crate) fn open<P: AsRef<OsStr>, S: AsRef<str>>(path: P, with: Option<S>) -> crate::Result<()> {
    match with {
        Some(program) => ::open::with_detached(path, program.as_ref()),
        None => ::open::that_detached(path),
    }
    .map_err(Into::into)
}

/// Opens URL with the program specified in `with`, or system default if `None`.
///
/// ## Platform-specific:
///
/// - **Android / iOS**: Always opens using default program.
///
/// # Examples
///
/// ```rust,no_run
/// tauri::Builder::default()
///   .setup(|app| {
///     // open the given URL on the system default browser
///     tauri_plugin_opener::open_url("https://github.com/tauri-apps/tauri", None::<&str>)?;
///     Ok(())
///   });
/// ```
pub fn open_url<P: AsRef<str>, S: AsRef<str>>(url: P, with: Option<S>) -> crate::Result<()> {
    let url = url.as_ref();
    open(url, with)
}

/// Opens path with the program specified in `with`, or system default if `None`.
///
/// ## Platform-specific:
///
/// - **Android / iOS**: Always opens using default program.
///
/// # Examples
///
/// ```rust,no_run
/// tauri::Builder::default()
///   .setup(|app| {
///     // open the given URL on the system default explorer
///     tauri_plugin_opener::open_path("/path/to/file", None::<&str>)?;
///     Ok(())
///   });
/// ```
pub fn open_path<P: AsRef<Path>, S: AsRef<str>>(path: P, with: Option<S>) -> crate::Result<()> {
    let path = path.as_ref();
    if with.is_none() {
        // Returns an IO error if not exists, and besides `exists()` is a shorthand for `metadata()`
        let metadata = path.metadata()?;

        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        if metadata.is_dir() {
            if let Ok(()) = open_dir_dbus(path) {
                return Ok(());
            }
        }
    }
    open(path, with)
}

/// Opens a dir with the default file manager via D-Bus.
#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn open_dir_dbus(path: &Path) -> crate::Result<()> {
    let connection = zbus::blocking::Connection::session()?;

    open_with_filemanager1(path, &connection)
        .or_else(|e| open_with_open_uri_portal(path, &connection).map_err(|_| e))
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn open_with_filemanager1(
    path: &Path,
    connection: &zbus::blocking::Connection,
) -> crate::Result<()> {
    let uri =
        url::Url::from_file_path(path).map_err(|_| crate::Error::FailedToConvertPathToFileUrl)?;

    #[zbus::proxy(
        interface = "org.freedesktop.FileManager1",
        default_service = "org.freedesktop.FileManager1",
        default_path = "/org/freedesktop/FileManager1"
    )]
    trait FileManager1 {
        async fn ShowFolders(&self, uris: Vec<&str>, startup_id: &str) -> crate::Result<()>;
    }

    let proxy = FileManager1ProxyBlocking::new(connection)?;

    proxy.ShowFolders(vec![uri.as_str()], "")
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
fn open_with_open_uri_portal(
    path: &Path,
    connection: &zbus::blocking::Connection,
) -> crate::Result<()> {
    use std::collections::HashMap;
    let uri =
        url::Url::from_file_path(path).map_err(|_| crate::Error::FailedToConvertPathToFileUrl)?;

    #[zbus::proxy(
        interface = "org.freedesktop.portal.Desktop",
        default_service = "org.freedesktop.portal.OpenURI",
        default_path = "/org/freedesktop/portal/desktop"
    )]
    trait PortalDesktop {
        async fn OpenDirectory(
            &self,
            parent_window: &str,
            uri: &str,
            options: HashMap<&str, &str>,
        ) -> crate::Result<()>;
    }

    let proxy = PortalDesktopProxyBlocking::new(connection)?;

    proxy.OpenDirectory("", uri.as_str(), HashMap::new())
}
