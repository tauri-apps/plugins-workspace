// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::Result;
use tauri::{command, AppHandle, Runtime};

#[cfg(target_os = "windows")]
use windows::{
    core::{HSTRING, PCWSTR},
    // CRITICAL FIX: Changed `Interface` to `ComInterface` so `.cast()` works
    Win32::{
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_INPROC_SERVER,
            COINIT_APARTMENTTHREADED,
        },
        UI::Shell::{
            ApplicationDestinations, ApplicationDocumentLists, Common::IObjectArray,
            IApplicationDestinations, IApplicationDocumentLists, IShellItem, SHAddToRecentDocs,
            SHCreateItemFromParsingName, ADLT_RECENT, SHARDAPPIDINFO, SHARD_APPIDINFO,
            SIGDN_FILESYSPATH,
        },
    },
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
        let _com_guard = ComGuard::new();

        let app_id = app.config().identifier.clone();
        let app_id_hstring = HSTRING::from(app_id);

        let path_hstring = HSTRING::from(_path);
        let item = SHCreateItemFromParsingName(&path_hstring, None)?;

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
        let _guard = ComGuard::new();

        let app_id = _app.config().identifier.clone();
        let app_id_hstring = HSTRING::from(app_id);

        if let Ok(dests) = CoCreateInstance::<_, IApplicationDestinations>(
            &ApplicationDestinations as *const _,
            None,
            CLSCTX_INPROC_SERVER,
        ) {
            let _ = dests.SetAppID(PCWSTR::from_raw(app_id_hstring.as_ptr()));
            let _ = dests.RemoveAllDestinations();
        }
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
        let _guard = ComGuard::new();

        let app_id = _app.config().identifier.clone();
        let app_id_hstring = HSTRING::from(app_id);
        let app_id_pcwstr = PCWSTR::from_raw(app_id_hstring.as_ptr());

        if let Ok(doc_lists) = CoCreateInstance::<_, IApplicationDocumentLists>(
            &ApplicationDocumentLists,
            None,
            CLSCTX_INPROC_SERVER,
        ) {
            let _ = doc_lists.SetAppID(app_id_pcwstr);
            if let Ok(obj_array) = doc_lists.GetList::<IObjectArray>(ADLT_RECENT, 30) {
                let count = obj_array.GetCount().unwrap_or(0);

                for i in 0..count {
                    if let Ok(shell_item) = obj_array.GetAt::<IShellItem>(i) {
                        if let Ok(name_pwstr) = shell_item.GetDisplayName(SIGDN_FILESYSPATH) {
                            if let Ok(path) = name_pwstr.to_string() {
                                if !path.is_empty() {
                                    recent_docs.push(path);
                                }
                            }

                            CoTaskMemFree(Some(name_pwstr.as_ptr() as *const core::ffi::c_void));
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
