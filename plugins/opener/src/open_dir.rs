// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

/// Open a path the system's default explorer.
///
/// ## Platform-specific:
///
/// - **Android / iOS:** Unsupported.
pub fn open_dir<P: AsRef<Path>>(path: P) -> crate::Result<()> {
    let path = dunce::canonicalize(path.as_ref())?;

    #[cfg(any(
        windows,
        target_os = "macos",
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    return imp::open_dir(&path);

    #[cfg(not(any(
        windows,
        target_os = "macos",
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    )))]
    Err(crate::Error::UnsupportedPlatform)
}

#[cfg(any(windows, target_os = "macos"))]
mod imp {
    use std::path::Path;

    pub fn open_dir(path: &Path) -> crate::Result<()> {
        crate::open::open_path(path, None::<&str>)
    }
}

#[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod imp {
    use std::collections::HashMap;
    use std::path::Path;

    pub fn open_dir(path: &Path) -> crate::Result<()> {
        let connection = zbus::blocking::Connection::session()?;

        open_with_filemanager1(path, &connection).or_else(|e| {
            open_with_open_uri_portal(path, &connection)
                .map_err(|_| e)
        })
    }

    fn open_with_filemanager1(
        path: &Path,
        connection: &zbus::blocking::Connection,
    ) -> crate::Result<()> {
        let uri = url::Url::from_file_path(path)
            .map_err(|_| crate::Error::FailedToConvertPathToFileUrl)?;

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

    fn open_with_open_uri_portal(
        path: &Path,
        connection: &zbus::blocking::Connection,
    ) -> crate::Result<()> {
        let uri = url::Url::from_file_path(path)
            .map_err(|_| crate::Error::FailedToConvertPathToFileUrl)?;

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
}
