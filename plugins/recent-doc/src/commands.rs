// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::Result;
use tauri::{command, AppHandle, Runtime};

#[cfg(target_os = "windows")]
use windows::{
    // CRITICAL FIX: Changed `Interface` to `ComInterface` so `.cast()` works
    Win32::{
        Foundation::MAX_PATH,
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance,
            CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile, STGM_READ,
        },
        UI::Shell::{
            FOLDERID_Recent, IShellLinkW, KNOWN_FOLDER_FLAG, SHARD_APPIDINFO, SHARDAPPIDINFO, SHAddToRecentDocs, SHCreateItemFromParsingName, SHGetKnownFolderPath, ShellLink
        },
    }, core::{HSTRING, Interface, PCWSTR, PWSTR}
};

#[cfg(target_os = "macos")]
use {
    objc2::MainThreadMarker,
    objc2_app_kit::NSDocumentController,
    objc2_foundation::{NSString, NSURL},
};

/// ComGuard ensures that the COM library is properly initialized and uninitialized on Windows.
#[cfg(target_os = "windows")]
struct ComGuard;

#[cfg(target_os = "windows")]
impl ComGuard {
    fn new() -> Self {
        unsafe {
            let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        }
        ComGuard
    }
}

#[cfg(target_os = "windows")]
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

#[command]
pub(crate) fn add_recent_document<R: Runtime>(app: AppHandle<R>, _path: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        // Set guard first
        let _com_guard = ComGuard::new();

        // Get app id
        let app_id = app.config().identifier.clone();
        let app_id_hstring = HSTRING::from(app_id);

        // Convert path to HSTRING
        let path_hstring = HSTRING::from(_path);
        let item = SHCreateItemFromParsingName(&path_hstring, None)?;

        // construct info
        let info = SHARDAPPIDINFO {
            pszAppID: PCWSTR::from_raw(app_id_hstring.as_ptr()),
            psi: std::mem::ManuallyDrop::new(Some(item)),
        };

        SHAddToRecentDocs(
            SHARD_APPIDINFO.0 as u32,
            Some(&info as *const _ as *const core::ffi::c_void),
        );
    }

    #[cfg(target_os = "macos")]
    {
        let ns_path = NSURL::fileURLWithPath(&NSString::from_str(_path));
        let mtm = MainThreadMarker::new().expect("AppKit API must be called on the main thread");
        let controller = NSDocumentController::sharedDocumentController(mtm);
        controller.noteNewRecentDocumentURL(&ns_path);
    }

    // CRITICAL FIX: Ensure this doesn't run on macOS
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Err(crate::error::Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(())
}

#[command]
pub(crate) fn clear_recent_documents<R: Runtime>(_app: AppHandle<R>) -> Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        let _com_guard = ComGuard::new();
        SHAddToRecentDocs(SHARD_APPIDINFO.0 as u32, None);
    }

    #[cfg(target_os = "macos")]
    unsafe {
        let mtm = MainThreadMarker::new().expect("AppKit API must be called on the main thread");
        let controller = NSDocumentController::sharedDocumentController(mtm);
        controller.clearRecentDocuments(None);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Err(crate::error::Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(())
}

#[command]
pub(crate) fn get_recent_documents<R: Runtime>(_app: AppHandle<R>) -> Result<Vec<String>> {
    #[allow(unused_mut)]
    let mut recent_docs = Vec::new();

    #[cfg(target_os = "windows")]
    unsafe {
        // Ensure COM is initialized for this worker thread
        let _guard = ComGuard::new();

        // Retrieve the absolute path to "%APPDATA%\Microsoft\Windows\Recent"
        let recent_path_pwstr = SHGetKnownFolderPath(&FOLDERID_Recent, KNOWN_FOLDER_FLAG(0), None)?;

        if !recent_path_pwstr.as_ptr().is_null() {
            let recent_path = recent_path_pwstr.to_string()?;
            // Free the string memory allocated by the Windows Shell
            CoTaskMemFree(Some(recent_path_pwstr.as_ptr() as *mut _));

            // Iterate through the hidden directory searching for .lnk files
            if let Ok(entries) = std::fs::read_dir(recent_path) {
                for entry in entries.flatten() {
                    let path = entry.path();

                    if path.extension().and_then(|s| s.to_str()) == Some("lnk") {
                        // Resolve the shortcut binary to extract its actual target path
                        if let Ok(resolved) = resolve_shortcut(&path) {
                            recent_docs.push(resolved);
                        }
                    }
                }
            }
        }
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

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Err(crate::error::Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(recent_docs)
}

/// Helper function to extract the real file system path from a Windows .lnk shortcut file.
#[cfg(target_os = "windows")]
fn resolve_shortcut(lnk_path: &std::path::Path) -> Result<String> {
    unsafe {
        // Instantiate the ShellLink COM Object
        let shell_link: IShellLinkW =
            CoCreateInstance(&ShellLink as *const _, None, CLSCTX_INPROC_SERVER)?;

        // Query the IPersistFile interface to load the shortcut binary from disk
        // This .cast() now compiles perfectly because ComInterface is imported
        let persist_file: IPersistFile = shell_link.cast()?;

        let path_hstring = HSTRING::from(lnk_path.as_os_str());
        persist_file.Load(&path_hstring, STGM_READ)?;

        // Allocate a buffer to hold the resolved target path
        let mut target_path = [0u16; MAX_PATH as usize];
        shell_link.GetPath(&mut target_path, std::ptr::null_mut(), 0)?;

        // Convert the UTF-16 C-string back to a standard Rust String
        let path_string = PWSTR::from_raw(target_path.as_mut_ptr()).to_string()?;

        Ok(path_string)
    }
}
