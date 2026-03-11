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
    return imp::reveal_items_in_dir(&[path]);

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
pub fn reveal_items_in_dir<I, P>(paths: I) -> crate::Result<()>
where
    I: IntoIterator<Item = P>,
    P: AsRef<Path>,
{
    let mut canonicalized = vec![];

    for path in paths {
        let path = dunce::canonicalize(path.as_ref())?;
        canonicalized.push(path);
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
    return imp::reveal_items_in_dir(&canonicalized);

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
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

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

    pub(super) fn normalize_path_for_shell(path: &Path) -> std::borrow::Cow<'_, Path> {
        if let Some(s) = path.to_str() {
            if let Some(unc) = s.strip_prefix(r"\\?\UNC\") {
                return std::borrow::Cow::Owned(PathBuf::from(format!(r"\\{}", unc)));
            }
        }
        std::borrow::Cow::Borrowed(path)
    }

    pub fn reveal_items_in_dir(paths: &[PathBuf]) -> crate::Result<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let mut grouped_paths: HashMap<&Path, Vec<&Path>> = HashMap::new();
        for path in paths {
            let parent = path
                .parent()
                .ok_or_else(|| crate::Error::NoParent(path.to_path_buf()))?;
            grouped_paths.entry(parent).or_default().push(path);
        }

        let _ = unsafe { CoInitialize(None) };

        for (parent, to_reveals) in grouped_paths {
            let parent_item_id_list = OwnedItemIdList::new(parent)?;
            let to_reveals_item_id_list = to_reveals
                .iter()
                .map(|to_reveal| OwnedItemIdList::new(to_reveal))
                .collect::<crate::Result<Vec<_>>>()?;
            if let Err(e) = unsafe {
                SHOpenFolderAndSelectItems(
                    parent_item_id_list.item,
                    Some(
                        &to_reveals_item_id_list
                            .iter()
                            .map(|item| item.item)
                            .collect::<Vec<_>>(),
                    ),
                    0,
                )
            } {
                // from https://github.com/electron/electron/blob/10d967028af2e72382d16b7e2025d243b9e204ae/shell/common/platform_util_win.cc#L302
                // On some systems, the above call mysteriously fails with "file not
                // found" even though the file is there.  In these cases, ShellExecute()
                // seems to work as a fallback (although it won't select the file).
                //
                // Note: we only handle the first file here if multiple of are present
                if e.code().0 == ERROR_FILE_NOT_FOUND.0 as i32 {
                    let first_path = to_reveals[0];
                    let is_dir = first_path.is_dir();
                    let mut info = SHELLEXECUTEINFOW {
                        cbSize: std::mem::size_of::<SHELLEXECUTEINFOW>() as _,
                        nShow: SW_SHOWNORMAL.0,
                        lpFile: PCWSTR(parent_item_id_list.hstring.as_ptr()),
                        lpClass: if is_dir { w!("folder") } else { PCWSTR::null() },
                        lpVerb: if is_dir {
                            w!("explore")
                        } else {
                            PCWSTR::null()
                        },
                        ..Default::default()
                    };

                    unsafe { ShellExecuteExW(&mut info) }?;
                }
            }
        }

        Ok(())
    }

    struct OwnedItemIdList {
        hstring: HSTRING,
        item: *const ITEMIDLIST,
    }

    impl OwnedItemIdList {
        fn new(path: &Path) -> crate::Result<Self> {
            let path = normalize_path_for_shell(path);
            let path_hstring = HSTRING::from(path.as_ref());
            let item_id_list = unsafe { ILCreateFromPathW(&path_hstring) };
            if item_id_list.is_null() {
                Err(crate::Error::FailedToConvertPathToItemIdList(
                    path.to_path_buf(),
                ))
            } else {
                Ok(Self {
                    hstring: path_hstring,
                    item: item_id_list,
                })
            }
        }
    }

    impl Drop for OwnedItemIdList {
        fn drop(&mut self) {
            if !self.item.is_null() {
                unsafe { ILFree(Some(self.item)) };
            }
        }
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

    pub fn reveal_items_in_dir(paths: &[PathBuf]) -> crate::Result<()> {
        unsafe {
            let mut urls = Vec::new();

            for path in paths {
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

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::imp::normalize_path_for_shell;
    use std::path::{Path, PathBuf};

    fn simulate_canonicalize_unc() -> PathBuf {
        PathBuf::from(r"\\?\UNC\server\share\file.mkv")
    }

    #[test]
    fn unc_path_should_not_have_extended_prefix() {
        let canonicalized = simulate_canonicalize_unc();
        let result = normalize_path_for_shell(&canonicalized);
        let s = result.as_ref().to_str().unwrap();
        assert!(
            !s.starts_with(r"\\?\UNC\"),
            "Path still has \\?\\UNC\\ prefix: {}",
            s
        );
    }

    #[test]
    fn unc_path_converts_correctly() {
        let input = Path::new(r"\\?\UNC\server\share\file.mkv");
        let result = normalize_path_for_shell(input);
        assert_eq!(result.as_ref(), Path::new(r"\\server\share\file.mkv"));
    }

    #[test]
    fn local_path_unchanged() {
        let input = Path::new(r"C:\path\to\file.txt");
        let result = normalize_path_for_shell(input);
        assert_eq!(result.as_ref(), input);
    }

    #[test]
    fn plain_unc_unchanged() {
        let input = Path::new(r"\\server\share\file.mkv");
        let result = normalize_path_for_shell(input);
        assert_eq!(result.as_ref(), input);
    }

    #[test]
    fn unc_path_with_spaces_converts_correctly() {
        let input = Path::new(r"\\?\UNC\server\my share\my file.mkv");
        let result = normalize_path_for_shell(input);
        assert_eq!(result.as_ref(), Path::new(r"\\server\my share\my file.mkv"));
    }

    #[test]
    fn local_verbatim_path_unchanged() {
        let input = Path::new(r"\\?\C:\path\to\file.txt");
        let result = normalize_path_for_shell(input);
        assert_eq!(result.as_ref(), input);
    }
}
