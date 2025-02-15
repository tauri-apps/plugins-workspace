// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

/// Reveal a path the system's default explorer.
///
/// ## Platform-specific:
///
/// - **Android / iOS:** Unsupported.
pub fn reveal_item_in_dir<P: AsRef<Path>>(path: P) -> crate::Result<()> {
    let path = path.as_ref().canonicalize()?;

    #[cfg(any(
        windows,
        target_os = "macos",
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    return imp::reveal_item_in_dir(&path);

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

#[cfg(windows)]
mod imp {
    use super::*;

    use windows::{
        core::{w, HSTRING, PCWSTR},
        Win32::{
            Foundation::ERROR_FILE_NOT_FOUND,
            System::Com::CoInitialize,
            UI::{
                Shell::{
                    ILCreateFromPathW, ILFree, SHOpenFolderAndSelectItems, ShellExecuteExW,
                    SHELLEXECUTEINFOW,
                },
                WindowsAndMessaging::SW_SHOWNORMAL,
            },
        },
    };

    pub fn reveal_item_in_dir(path: &Path) -> crate::Result<()> {
        let file = dunce::simplified(path);

        let _ = unsafe { CoInitialize(None) };

        let dir = file
            .parent()
            .ok_or_else(|| crate::Error::NoParent(file.to_path_buf()))?;

        let dir = HSTRING::from(dir);
        let dir_item = unsafe { ILCreateFromPathW(&dir) };

        let file_h = HSTRING::from(file);
        let file_item = unsafe { ILCreateFromPathW(&file_h) };

        unsafe {
            if let Err(e) = SHOpenFolderAndSelectItems(dir_item, Some(&[file_item]), 0) {
                // from https://github.com/electron/electron/blob/10d967028af2e72382d16b7e2025d243b9e204ae/shell/common/platform_util_win.cc#L302
                // On some systems, the above call mysteriously fails with "file not
                // found" even though the file is there.  In these cases, ShellExecute()
                // seems to work as a fallback (although it won't select the file).
                if e.code().0 == ERROR_FILE_NOT_FOUND.0 as i32 {
                    let is_dir = file.is_dir();
                    let mut info = SHELLEXECUTEINFOW {
                        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as _,
                        nShow: SW_SHOWNORMAL.0,
                        lpFile: PCWSTR(dir.as_ptr()),
                        lpClass: if is_dir { w!("folder") } else { PCWSTR::null() },
                        lpVerb: if is_dir {
                            w!("explore")
                        } else {
                            PCWSTR::null()
                        },
                        ..std::mem::zeroed()
                    };

                    ShellExecuteExW(&mut info).inspect_err(|_| {
                        ILFree(Some(dir_item));
                        ILFree(Some(file_item));
                    })?;
                }
            }
        }

        unsafe {
            ILFree(Some(dir_item));
            ILFree(Some(file_item));
        }

        Ok(())
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

    use super::*;

    pub fn reveal_item_in_dir(path: &Path) -> crate::Result<()> {
        let connection = zbus::blocking::Connection::session()?;

        reveal_with_filemanager1(path, &connection)
            .or_else(|_| reveal_with_open_uri_portal(path, &connection))
    }

    fn reveal_with_filemanager1(
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
            async fn ShowItems(&self, name: Vec<&str>, arg2: &str) -> crate::Result<()>;
        }

        let proxy = FileManager1ProxyBlocking::new(connection)?;

        proxy.ShowItems(vec![uri.as_str()], "")
    }

    fn reveal_with_open_uri_portal(
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
                arg1: &str,
                name: &str,
                arg3: HashMap<&str, &str>,
            ) -> crate::Result<()>;
        }

        let proxy = PortalDesktopProxyBlocking::new(connection)?;

        proxy.OpenDirectory("", uri.as_str(), HashMap::new())
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use super::*;
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSArray, NSString, NSURL};
    pub fn reveal_item_in_dir(path: &Path) -> crate::Result<()> {
        unsafe {
            let path = path.to_string_lossy();
            let path = NSString::from_str(&path);
            let urls = vec![NSURL::fileURLWithPath(&path)];
            let urls = NSArray::from_vec(urls);

            let workspace = NSWorkspace::new();
            workspace.activateFileViewerSelectingURLs(&urls);
        }

        Ok(())
    }
}
