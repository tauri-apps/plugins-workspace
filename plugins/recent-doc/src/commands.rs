// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::Result;
use std::fs;
use tauri::command;

#[cfg(target_os = "windows")]
use windows::{
    core::{Interface, HSTRING, PWSTR},
    Win32::{
        Foundation::{GetLastError, MAX_PATH},
        System::Com::{
            CoCreateInstance, CoTaskMemFree, IPersistFile, CLSCTX_INPROC_SERVER, STGM_READ,
        },
        UI::Shell::{FOLDERID_Recent, SHGetKnownFolderPath, KNOWN_FOLDER_FLAG},
        UI::Shell::{IShellLinkW, SHAddToRecentDocs, ShellLink, SHARD_APPIDINFO, SHARD_PATHW},
    },
};

#[cfg(target_os = "macos")]
use {
    objc2::MainThreadMarker,
    objc2_app_kit::NSDocumentController,
    objc2_foundation::{NSString, NSURL},
};

#[command]
pub(crate) fn add_recent_document(_path: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        // Convert path to HSTRING
        let path_hstring = HSTRING::from(_path);
        SHAddToRecentDocs(
            SHARD_PATHW.0 as u32,
            Some(path_hstring.as_ptr() as *const core::ffi::c_void),
        );
    }

    #[cfg(target_os = "macos")]
    {
        let ns_path = NSURL::fileURLWithPath(&NSString::from_str(_path));
        let mtm = MainThreadMarker::new().expect("AppKit API must be called on the main thread");
        let controller = NSDocumentController::sharedDocumentController(mtm);
        controller.noteNewRecentDocumentURL(&ns_path);
    }

    #[cfg(unix)]
    {
        // Recent documents are not supported on Unix-like systems.
        Err(crate::error::Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(())
}

#[command]
pub(crate) fn clear_recent_documents() -> Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        SHAddToRecentDocs(SHARD_APPIDINFO.0 as u32, None);
    }

    #[cfg(target_os = "macos")]
    unsafe {
        let mtm = MainThreadMarker::new().expect("AppKit API must be called on the main thread");
        let controller = NSDocumentController::sharedDocumentController(mtm);
        controller.clearRecentDocuments(None);
    }

    #[cfg(unix)]
    {
        // Recent documents are not supported on Unix-like systems.
        Err(crate::error::Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(())
}

#[command]
pub(crate) fn get_recent_documents() -> Result<Vec<String>> {
    #[allow(unused_mut)]
    let mut recent_docs = Vec::new();

    #[cfg(target_os = "windows")]
    unsafe {
        let recent_path = SHGetKnownFolderPath(&FOLDERID_Recent, KNOWN_FOLDER_FLAG(0), None)?;
        if recent_path.is_null() {
            return Err(crate::error::Error::WindowsError(GetLastError().into()));
        }
        let recent_path_os_string = recent_path.to_hstring().to_os_string();

        if let Ok(entries) = fs::read_dir(recent_path_os_string) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.extension().and_then(|s: &std::ffi::OsStr| s.to_str()) == Some("lnk") {
                    if let Ok(resolved_path) = resolve_shortcut(&path) {
                        recent_docs.push(resolved_path);
                    }
                }
            }
        }

        CoTaskMemFree(Some(recent_path.0 as *mut _));
    }

    #[cfg(target_os = "macos")]
    {
        let mtm = MainThreadMarker::new().expect("AppKit API must be called on the main thread");
        let controller = NSDocumentController::sharedDocumentController(mtm);
        let urls = controller.recentDocumentURLs();

        for url in &*urls {
            if let Some(ns_path) = url.path() {
                recent_docs.push(ns_path.to_string());
            }
        }
    }

    #[cfg(unix)]
    {
        // Recent documents are not supported on Unix-like systems.
        Err(crate::error::Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(recent_docs)
}

#[cfg(target_os = "windows")]
fn resolve_shortcut(lnk_path: &std::path::Path) -> Result<String> {
    unsafe {
        // Create IShellLink instance
        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;

        // Get IPersistFile interface
        let persist_file: IPersistFile = shell_link.cast()?;

        // Convert path to wide string
        let path_wide = HSTRING::from(lnk_path);

        // Load the shortcut file
        persist_file.Load(&path_wide, STGM_READ)?;

        // Resolve the target path
        let mut target_path = [0u16; MAX_PATH as usize];
        shell_link.GetPath(&mut target_path, std::ptr::null_mut(), 0)?;

        // Convert wide string to regular string
        let path_string = PWSTR::from_raw(target_path.as_mut_ptr()).to_string()?;

        Ok(path_string)
    }
}
