// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

/// Reveal a path in the system's default explorer.
///
/// ## Platform-specific:
///
/// - **Android / iOS:** Unsupported.
pub fn reveal_item_in_dir<P: AsRef<Path>>(path: P) -> crate::Result<()> {
    let _path = canonicalize(path.as_ref())?;

    #[cfg(any(
        windows,
        target_os = "macos",
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    return imp::reveal_items_in_dir(&[_path]);

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

/// Reveal multiple paths in the system's default explorer.
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
        let path = canonicalize(path.as_ref())?;
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

fn canonicalize(path: &Path) -> crate::Result<PathBuf> {
    #[cfg(windows)]
    let path = crate::windows_shell_path::absolute_and_check_exists(dunce::simplified(path))?;
    #[cfg(not(windows))]
    let path = std::fs::canonicalize(path)?;
    Ok(path)
}

#[cfg(windows)]
mod imp {
    use std::{
        borrow::Cow,
        collections::HashMap,
        path::{Path, PathBuf},
    };

    use windows::Win32::UI::Shell::Common::ITEMIDLIST;
    use windows::{
        Win32::{
            Foundation::ERROR_FILE_NOT_FOUND,
            System::Com::CoInitialize,
            UI::{
                Shell::{
                    ILCreateFromPathW, ILFree, SHELLEXECUTEINFOW, SHOpenFolderAndSelectItems,
                    ShellExecuteExW,
                },
                WindowsAndMessaging::SW_SHOWNORMAL,
            },
        },
        core::{HSTRING, PCWSTR, w},
    };

    pub fn reveal_items_in_dir(paths: &[PathBuf]) -> crate::Result<()> {
        if paths.is_empty() {
            return Ok(());
        }

        let mut grouped_paths: HashMap<Cow<Path>, Vec<&Path>> = HashMap::new();
        for path in paths {
            let parent = crate::windows_shell_path::shell_parent_path(path)
                .ok_or_else(|| crate::Error::NoParent(path.to_path_buf()))?;
            grouped_paths.entry(parent).or_default().push(path);
        }

        let _ = unsafe { CoInitialize(None) };

        for (parent, to_reveals) in grouped_paths {
            let parent_item_id_list = OwnedItemIdList::new(&parent)?;
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
            let path_hstring = HSTRING::from(path);
            let item_id_list = unsafe { ILCreateFromPathW(&path_hstring) };
            if item_id_list.is_null() {
                Err(crate::Error::FailedToConvertPathToItemIdList(
                    path.to_owned(),
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
        // `zbus::blocking` starts its own tokio runtime when zbus is built with the `tokio` feature,
        // which panics with "Cannot start a runtime from within a runtime" if the caller is already
        // inside one (e.g. an async command), so always run it on a dedicated thread.
        std::thread::scope(|s| {
            s.spawn(|| reveal_items_in_dir_blocking(paths))
                .join()
                .unwrap_or_else(|e| std::panic::resume_unwind(e))
        })
    }

    fn reveal_items_in_dir_blocking(paths: &[PathBuf]) -> crate::Result<()> {
        let connection = zbus::blocking::Connection::session()?;

        reveal_items_with_connection(paths, &connection)
    }

    fn reveal_items_with_connection(
        paths: &[PathBuf],
        connection: &zbus::blocking::Connection,
    ) -> crate::Result<()> {
        reveal_with_filemanager1(paths, connection).or_else(|e| {
            // Fallback to opening the directory of the first item if revealing multiple items fails.
            if let Some(first_path) = paths.first() {
                reveal_with_open_uri_portal(first_path, connection)
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
        let file = open_portal_file(path)?;

        #[zbus::proxy(
            interface = "org.freedesktop.portal.OpenURI",
            default_service = "org.freedesktop.portal.Desktop",
            default_path = "/org/freedesktop/portal/desktop"
        )]
        trait PortalDesktop {
            async fn OpenDirectory(
                &self,
                parent_window: &str,
                fd: zbus::zvariant::Fd<'_>,
                options: HashMap<&str, zbus::zvariant::Value<'_>>,
            ) -> zbus::Result<zbus::zvariant::OwnedObjectPath>;
        }

        let proxy = PortalDesktopProxyBlocking::new(connection)?;

        proxy.OpenDirectory("", zbus::zvariant::Fd::from(&file), HashMap::new())?;
        Ok(())
    }
    fn open_portal_file(path: &Path) -> std::io::Result<std::fs::File> {
        use std::os::unix::fs::OpenOptionsExt;
        std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(path)
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::io::{BufRead, BufReader};
        use std::process::{Child, Command, Stdio};
        use std::sync::Arc;
        use std::sync::atomic::{AtomicBool, Ordering};
        use std::sync::mpsc::{self, Sender};
        use std::time::Duration;

        struct PrivateBus(Child);

        impl Drop for PrivateBus {
            fn drop(&mut self) {
                let _ = self.0.kill();
                let _ = self.0.wait();
            }
        }

        struct FallbackFileManager {
            events: Sender<&'static str>,
            fail: Arc<AtomicBool>,
        }

        #[zbus::interface(name = "org.freedesktop.FileManager1")]
        impl FallbackFileManager {
            fn show_items(&self, _uris: Vec<String>, _startup_id: String) -> zbus::fdo::Result<()> {
                self.events.send("primary").unwrap();
                if self.fail.load(Ordering::SeqCst) {
                    Err(zbus::fdo::Error::Failed("primary fixture error".into()))
                } else {
                    Ok(())
                }
            }
        }

        struct Portal {
            events: Sender<&'static str>,
            received: Sender<(String, String, usize)>,
            fail: Arc<AtomicBool>,
        }

        #[zbus::interface(name = "org.freedesktop.portal.OpenURI")]
        impl Portal {
            fn open_directory(
                &self,
                parent_window: String,
                fd: zbus::zvariant::OwnedFd,
                options: HashMap<String, zbus::zvariant::OwnedValue>,
            ) -> zbus::fdo::Result<zbus::zvariant::OwnedObjectPath> {
                use std::io::Read;
                self.events.send("portal").unwrap();
                let mut file = std::fs::File::from(std::os::fd::OwnedFd::from(fd));
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                self.received
                    .send((parent_window, contents, options.len()))
                    .unwrap();
                if self.fail.load(Ordering::SeqCst) {
                    Err(zbus::fdo::Error::Failed("portal fixture error".into()))
                } else {
                    Ok(zbus::zvariant::OwnedObjectPath::try_from(
                        "/org/freedesktop/portal/desktop/request/fixture",
                    )
                    .unwrap())
                }
            }
        }

        struct TestDirectory(PathBuf);

        impl Drop for TestDirectory {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }

        #[test]
        fn portal_fallback_transfers_fd_and_preserves_order_and_errors() {
            let directory = TestDirectory(std::env::temp_dir().join(format!(
                "opener-portal-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
            std::fs::create_dir(&directory.0).unwrap();
            let paths = [
                directory.0.join("first file\nλ"),
                directory.0.join("second"),
            ];
            std::fs::write(&paths[0], "first file contents").unwrap();
            std::fs::write(&paths[1], "second file contents").unwrap();
            let mut bus = PrivateBus(
                Command::new("dbus-daemon")
                    .args(["--session", "--nofork", "--print-address=1"])
                    .stdout(Stdio::piped())
                    .spawn()
                    .expect("dbus-daemon is required for the D-Bus integration test"),
            );
            let mut address = String::new();
            BufReader::new(bus.0.stdout.take().unwrap())
                .read_line(&mut address)
                .unwrap();
            let (events_tx, events_rx) = mpsc::channel();
            let (received_tx, received_rx) = mpsc::channel();
            let primary_fails = Arc::new(AtomicBool::new(false));
            let portal_fails = Arc::new(AtomicBool::new(false));

            {
                let _primary = zbus::blocking::connection::Builder::address(address.trim())
                    .unwrap()
                    .name("org.freedesktop.FileManager1")
                    .unwrap()
                    .serve_at(
                        "/org/freedesktop/FileManager1",
                        FallbackFileManager {
                            events: events_tx.clone(),
                            fail: primary_fails.clone(),
                        },
                    )
                    .unwrap()
                    .build()
                    .unwrap();
                let _portal = zbus::blocking::connection::Builder::address(address.trim())
                    .unwrap()
                    .name("org.freedesktop.portal.Desktop")
                    .unwrap()
                    .serve_at(
                        "/org/freedesktop/portal/desktop",
                        Portal {
                            events: events_tx,
                            received: received_tx,
                            fail: portal_fails.clone(),
                        },
                    )
                    .unwrap()
                    .build()
                    .unwrap();
                let client = zbus::blocking::connection::Builder::address(address.trim())
                    .unwrap()
                    .build()
                    .unwrap();

                reveal_items_with_connection(&paths, &client).unwrap();
                assert_eq!(
                    events_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
                    "primary"
                );
                assert!(
                    events_rx.try_recv().is_err(),
                    "no fallback on primary success"
                );

                primary_fails.store(true, Ordering::SeqCst);
                reveal_items_with_connection(&paths, &client).unwrap();
                assert_eq!(
                    events_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
                    "primary"
                );
                assert_eq!(
                    events_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
                    "portal"
                );
                assert_eq!(
                    received_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
                    (String::new(), "first file contents".into(), 0)
                );

                portal_fails.store(true, Ordering::SeqCst);
                let error = reveal_items_with_connection(&paths, &client).unwrap_err();
                assert!(error.to_string().contains("portal fixture error"));
                assert_eq!(
                    events_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
                    "primary"
                );
                assert_eq!(
                    events_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
                    "portal"
                );
                received_rx.recv_timeout(Duration::from_secs(5)).unwrap();

                let missing = [directory.0.join("missing")];
                assert!(matches!(
                    reveal_items_with_connection(&missing, &client),
                    Err(crate::Error::Io(_))
                ));
                assert_eq!(
                    events_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
                    "primary"
                );
                assert!(
                    events_rx.try_recv().is_err(),
                    "do not call the portal without a file"
                );
                assert!(reveal_items_with_connection(&[], &client).is_err());
                assert_eq!(
                    events_rx.recv_timeout(Duration::from_secs(5)).unwrap(),
                    "primary"
                );
                assert!(
                    events_rx.try_recv().is_err(),
                    "no first path means no fallback"
                );
            }
        }
        #[test]
        fn maintained_dbus_portal_file_open_does_not_wait_for_a_fifo_writer() {
            use std::os::unix::fs::{FileTypeExt, OpenOptionsExt};
            let directory = TestDirectory(std::env::temp_dir().join(format!(
                "opener-fifo-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            )));
            std::fs::create_dir(&directory.0).unwrap();
            let fifo = directory.0.join("without-writer");
            assert!(
                Command::new("mkfifo")
                    .arg(&fifo)
                    .status()
                    .unwrap()
                    .success()
            );
            let path = fifo.clone();
            let (tx, rx) = mpsc::channel();
            let probe = std::thread::spawn(move || {
                let _ = tx.send(open_portal_file(&path));
            });
            let result = rx.recv_timeout(Duration::from_secs(5));
            if result.is_err() {
                // Release an incorrectly blocking reader before reporting the bounded failure.
                let _writer = std::fs::OpenOptions::new()
                    .write(true)
                    .custom_flags(libc::O_NONBLOCK)
                    .open(&fifo);
                let _ = rx.recv_timeout(Duration::from_secs(5));
            }
            let file = result
                .expect("opening a portal FIFO must not wait for a writer")
                .unwrap();
            probe.join().unwrap();
            assert!(file.metadata().unwrap().file_type().is_fifo());
        }
    }
}

#[cfg(target_os = "macos")]
mod imp {
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSArray, NSString, NSURL};
    use std::path::PathBuf;

    pub fn reveal_items_in_dir(paths: &[PathBuf]) -> crate::Result<()> {
        // these AppKit calls are only `unsafe` before objc2-app-kit 0.3.2
        #[allow(unused_unsafe)]
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
