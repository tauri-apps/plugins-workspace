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

/// Reveal the paths the system's default explorer.
///
/// ## Platform-specific:
///
/// - **Android / iOS:** Unsupported.
pub fn reveal_items_in_dir<P: AsRef<Path>>(paths: &Vec<P>) -> crate::Result<()> {
    let mut path_bufs = vec![];

    for path in paths.iter() {
        let path = path.as_ref().canonicalize()?;
        path_bufs.push(path);
    }

    #[cfg(any(
        windows,
        target_os = "macos",
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    return imp::reveal_items_in_dir(&path_bufs);

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
    use std::path::Path;

    use windows::Win32::UI::Shell::Common::ITEMIDLIST;
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

    pub fn reveal_item_in_dir<P: AsRef<Path>>(path: P) -> crate::Result<()> {
        let file = dunce::simplified(path);

        let _ = unsafe { CoInitialize(None) };

        let dir = file
            .parent()
            .ok_or_else(|| crate::Error::NoParent(file.to_path_buf()))?;

        let dir_h = HSTRING::from(dir);
        let dir_item = unsafe { ILCreateFromPathW(&dir_h) };

        let file_h = HSTRING::from(file.as_os_str());
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
                        lpFile: PCWSTR(dir_h.as_ptr()),
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

    pub fn reveal_items_in_dir<P: AsRef<Path>>(paths: &Vec<P>) -> crate::Result<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let first_path = dunce::simplified(&paths[0]);
        let dir = first_path
            .parent()
            .ok_or_else(|| crate::Error::NoParent(first_path.to_path_buf()))?;

        let _ = unsafe { CoInitialize(None) };

        let dir_h = HSTRING::from(dir);
        let dir_item = unsafe { ILCreateFromPathW(&dir_h) };

        let mut items_to_free: Vec<*const ITEMIDLIST> = Vec::with_capacity(paths.len() + 1);
        items_to_free.push(dir_item);

        let mut file_items: Vec<*const ITEMIDLIST> = Vec::with_capacity(paths.len());

        for path in paths {
            let simplified_path = dunce::simplified(path);
            if simplified_path.parent() != Some(dir) {
                // All items must be in the same directory.
                // You might want to return an error here.
                continue;
            }
            let file_h = HSTRING::from(simplified_path.as_os_str());
            let file_item = unsafe { ILCreateFromPathW(&file_h) };
            file_items.push(file_item);
            items_to_free.push(file_item);
        }

        let result = unsafe {
            SHOpenFolderAndSelectItems(dir_item, Some(&file_items), 0).map_err(Into::into)
        };

        for item in items_to_free {
            unsafe { ILFree(Some(item)) };
        }

        result
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
    use super::*;
    use std::collections::HashMap;

    pub fn reveal_item_in_dir<P: AsRef<Path>>(path: P) -> crate::Result<()> {
        let connection = zbus::blocking::Connection::session()?;

        reveal_with_filemanager1(&vec![path], &connection)
            .or_else(|_| reveal_with_open_uri_portal(path, &connection))
    }

    pub fn reveal_items_in_dir<P: AsRef<Path>>(paths: &Vec<P>) -> crate::Result<()> {
        let connection = zbus::blocking::Connection::session()?;

        reveal_with_filemanager1(paths, &connection).or_else(|e| {
            // Fallback to opening the directory of the first item if revealing multiple items fails.
            if let Some(first_path) = paths.first() {
                reveal_with_open_uri_portal(first_path, &connection)
            } else {
                Err(e)
            }
        })
    }

    fn reveal_with_filemanager1<P: AsRef<Path>>(
        paths: &Vec<P>,
        connection: &zbus::blocking::Connection,
    ) -> crate::Result<()> {
        let uris: Result<Vec<_>, _> = paths
            .iter()
            .map(|path| {
                url::Url::from_file_path(path)
                    .map_err(|_| crate::Error::FailedToConvertPathToFileUrl)
            })
            .collect();
        let uris = uris?;
        let uri_strs: Vec<&str> = uris.iter().map(|uri| uri.as_str()).collect();

        #[zbus::proxy(
            interface = "org.freedesktop.FileManager1",
            default_service = "org.freedesktop.FileManager1",
            default_path = "/org/freedesktop/FileManager1"
        )]
        trait FileManager1 {
            async fn ShowItems(&self, name: Vec<&str>, arg2: &str) -> crate::Result<()>;
        }

        let proxy = FileManager1ProxyBlocking::new(connection)?;

        proxy.ShowItems(uri_strs, "")
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

    pub fn reveal_item_in_dir<P: AsRef<Path>>(path: P) -> crate::Result<()> {
        unsafe {
            let path = path.as_ref().to_string_lossy();
            let path = NSString::from_str(&path);
            let urls = vec![NSURL::fileURLWithPath(&path)];
            let urls = NSArray::from_retained_slice(&urls);

            let workspace = NSWorkspace::new();
            workspace.activateFileViewerSelectingURLs(&urls);
        }

        Ok(())
    }

    pub fn reveal_items_in_dir<P: AsRef<Path>>(paths: &Vec<P>) -> crate::Result<()> {
        unsafe {
            let mut urls = Vec::new();

            for path in paths.iter() {
                let path = path.as_ref().to_string_lossy();
                let path = NSString::from_str(&path);
                let url = NSURL::fileURLWithPath(&path);

                urls.push(url);
            }

            let urls = NSArray::from_retained_slice(&urls);

            let workspace = NSWorkspace::new();
            workspace.activateFileViewerSelectingURLs(&urls);
        }

        Ok(())
    }
}
