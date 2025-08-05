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
    use std::path::PathBuf;

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

    pub fn reveal_item_in_dir(path: &PathBuf) -> crate::Result<()> {
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

    pub fn reveal_items_in_dir(paths: &[PathBuf]) -> crate::Result<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let first_path = dunce::simplified(&paths[0]);
        let parent_dir = first_path
            .parent()
            .ok_or_else(|| crate::Error::NoParent(first_path.to_path_buf()))?;

        // On Windows, SHOpenFolderAndSelectItems requires all items to be in the same directory.
        // We filter the paths to ensure they all share the same parent as the first path.
        let files_in_same_dir: Vec<_> = paths
            .iter()
            .map(|p| dunce::simplified(p))
            .filter(|p| p.parent() == Some(parent_dir))
            .collect();

        if files_in_same_dir.is_empty() {
            // This case can happen if the original list had paths from different directories.
            // We can't open multiple directories, so we do nothing.
            return Ok(());
        }

        let _ = unsafe { CoInitialize(None) };

        let dir_hstring = HSTRING::from(parent_dir);
        let dir_item = unsafe { ILCreateFromPathW(&dir_hstring) };

        // Ensure dir_item is freed even if subsequent operations fail.
        let mut created_file_items = Vec::new();

        for path in &files_in_same_dir {
            let file_hstring = HSTRING::from(path.as_os_str());
            let file_item = unsafe { ILCreateFromPathW(&file_hstring) };
            if !file_item.is_null() {
                created_file_items.push(file_item);
            }
        }

        // The function expects a slice of *const ITEMIDLIST, so we must cast our *mut pointers.
        let item_id_lists_const: Vec<*const ITEMIDLIST> = created_file_items
            .iter()
            .map(|&p| p as *const _)
            .collect();

        let result = unsafe {
            if let Err(e) = SHOpenFolderAndSelectItems(dir_item, Some(&item_id_lists_const), 0) {
                // Fallback logic from the original function.
                if e.code().0 == ERROR_FILE_NOT_FOUND.0 as i32 {
                    let mut info = SHELLEXECUTEINFOW {
                        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as _,
                        nShow: SW_SHOWNORMAL.0,
                        lpFile: PCWSTR(dir_hstring.as_ptr()),
                        lpVerb: w!("explore"),
                        ..Default::default()
                    };
                    ShellExecuteExW(&mut info).map(|_| ()).map_err(Into::into)
                } else {
                    Err(e.into())
                }
            } else {
                Ok(())
            }
        };

        // Free all allocated ITEMIDLISTs
        unsafe {
            for item in created_file_items {
                ILFree(Some(item));
            }
            ILFree(Some(dir_item));
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
    use std::path::PathBuf;

    pub fn reveal_item_in_dir(path: &PathBuf) -> crate::Result<()> {
        let connection = zbus::blocking::Connection::session()?;

        reveal_with_filemanager1(&[path.clone()], &connection)
            .or_else(|_| reveal_with_open_uri_portal(&path, &connection))
    }

    pub fn reveal_items_in_dir(paths: &[PathBuf]) -> crate::Result<()> {
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

    fn reveal_with_filemanager1(
        paths: &[PathBuf],
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
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSArray, NSString, NSURL};
    use std::path::PathBuf;

    pub fn reveal_item_in_dir(path: &PathBuf) -> crate::Result<()> {
        unsafe {
            let path = path.to_string_lossy();
            let path = NSString::from_str(&path);
            let urls = vec![NSURL::fileURLWithPath(&path)];
            let urls = NSArray::from_retained_slice(&urls);

            let workspace = NSWorkspace::new();
            workspace.activateFileViewerSelectingURLs(&urls);
        }

        Ok(())
    }

    pub fn reveal_items_in_dir(paths: &[PathBuf]) -> crate::Result<()> {
        unsafe {
            let mut urls = Vec::new();

            for path in paths.iter() {
                let path = path.to_string_lossy();
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
