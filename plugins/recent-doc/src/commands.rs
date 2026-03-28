// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::{Error, Result};
use tauri::command;

#[command]
/// add recent
pub(crate) fn add_recent_document(path: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use windows::{
            core::HSTRING,
            Win32::System::Com::GetAppUserModelId,
            Win32::UI::Shell::{
                SHAddToRecentDocs, SHCreateItemFromParsingName, SHARDAPPIDINFO, SHARD_APPIDINFO,
            },
        };
        unsafe {
            let path_hstring = HSTRING::from(path);
            let item = SHCreateItemFromParsingName(&path_hstring, None).ok()?;

            let info: SHARDAPPIDINFO = SHARDAPPIDINFO {
                psi: Some(item),
                pszAppID: GetAppUserModelId()?,
            };

            SHAddToRecentDocs(SHARD_APPIDINFO, &info as *const _);
        }
    }

    #[cfg(target_os = "macos")]
    {
        use objc2::rc::Id;
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSDocumentController;
        use objc2_foundation::{NSString, NSURL};

        unsafe {
            let ns_path = NSURL::fileURLWithPath(&NSString::from_str(path));
            let mtm =
                MainThreadMarker::new().expect("AppKit API must be called on the main thread");
            let controller: Id<NSDocumentController> =
                NSDocumentController::sharedDocumentController(mtm);
            controller.noteNewRecentDocumentURL(&ns_path);
        }
    }

    #[cfg(unix)]
    {
        // Recent documents are not supported on Unix-like systems.
        Err(Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(())
}

#[command]
pub(crate) fn clear_recent_documents() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::UI::Shell::{SHAddToRecentDocs, SHARD_APPIDINFO};
        unsafe {
            SHAddToRecentDocs(SHARD_APPIDINFO, std::ptr::null());
        }
    }

    #[cfg(target_os = "macos")]
    {
        use objc2::rc::Id;
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSDocumentController;

        unsafe {
            let mtm =
                MainThreadMarker::new().expect("AppKit API must be called on the main thread");
            let controller: Id<NSDocumentController> =
                NSDocumentController::sharedDocumentController(mtm);
            controller.clearRecentDocuments(None);
        }
    }

    #[cfg(unix)]
    {
        // Recent documents are not supported on Unix-like systems.
        Err(Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(())
}

#[command]
pub(crate) fn get_recent_documents() -> Result<Vec<String>> {
    let mut recent_docs = Vec::new();

    #[cfg(target_os = "windows")]
    {
        use std::fs;
        use std::path::{Path, PathBuf};
        use windows::{
            core::PWSTR,
            Win32::System::Com::CoTaskMemFree,
            Win32::UI::Shell::{FOLDERID_Recent, SHGetKnownFolderPath},
        };
        unsafe {
            let recent_path_ptr: PWSTR = SHGetKnownFolderPath(&FOLDERID_Recent, 0, None)?;

            if !recent_path_ptr.is_null() {
                let recent_path = PWSTR::from_raw(recent_path_ptr.0);
                let recent_os_string = recent_path.to_string();
                let recent_folder = PathBuf::from(recent_os_string);

                if let Ok(entries) = fs::read_dir(recent_folder) {
                    for entry in entries.flatten() {
                        if let Ok(entry) = entry {
                            let path = entry.path();

                            if path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                                if let Ok(resolved_path) = Self::resolve_shortcut(&path) {
                                    recent_docs.push(resolved_path);
                                }
                            }
                        }
                    }
                }

                CoTaskMemFree(recent_path_ptr.0 as *mut _);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        use objc2::rc::Id;
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSDocumentController;
        use objc2_foundation::{NSArray, NSString};

        unsafe {
            let mtm =
                MainThreadMarker::new().expect("AppKit API must be called on the main thread");
            let controller: Id<NSDocumentController> =
                NSDocumentController::sharedDocumentController(mtm);
            let urls = controller.recentDocumentURLs();

            for url in &*urls {
                if let Some(ns_path) = url.path() {
                    recent_docs.push(ns_path.to_string());
                }
            }
        }
    }

    #[cfg(unix)]
    {
        // Recent documents are not supported on Unix-like systems.
        Err(Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(recent_docs)
}

#[cfg(target_os = "windows")]
fn resolve_shortcut(lnk_path: &Path) -> Result<String> {
    use windows::{
        core::{ComInterface, HSTRING, PWSTR},
        Win32::Foundation::MAX_PATH,
        Win32::System::Com::{CoCreateInstance, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ},
        Win32::UI::Shell::{IShellLinkW, ShellLink},
    };

    unsafe {
        // Create IShellLink instance
        let shell_link: IShellLinkW =
            CoCreateInstance(&ShellLink as *const _, None, CLSCTX_INPROC_SERVER)?;

        // Get IPersistFile interface
        let persist_file: IPersistFile = shell_link.cast()?;

        // Convert path to wide string
        let path_wide = HSTRING::from(lnk_path);

        // Load the shortcut file
        persist_file.Load(&path_wide, STGM_READ)?;

        // Resolve the target path
        let mut target_path = [0u16; MAX_PATH as usize];
        shell_link.GetPath(&mut target_path, None, None, 0)?;

        // Convert wide string to regular string
        let path_string = PWSTR::from_raw(target_path.as_mut_ptr()).to_string()?;

        Ok(path_string)
    }
}
