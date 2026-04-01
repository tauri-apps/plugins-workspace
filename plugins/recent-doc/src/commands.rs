// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::Result;
use tauri::{command, AppHandle, Runtime};

#[cfg(target_os = "windows")]
use windows::{
    core::{HSTRING, PCWSTR},
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
pub(crate) fn add_recent_document<R: Runtime>(_app: AppHandle<R>, _path: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        let _com_guard = ComGuard::new();

        let app_id = &_app.config().identifier;
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

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Err(crate::error::Error::UnsupportedPlatform)?;
    }

    Ok(())
}

#[command]
pub(crate) fn clear_recent_documents<R: Runtime>(_app: AppHandle<R>) -> Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        let _guard = ComGuard::new();

        let app_id = &_app.config().identifier;
        let app_id_hstring = HSTRING::from(app_id);

        let dests: IApplicationDestinations =
            CoCreateInstance(&ApplicationDestinations, None, CLSCTX_INPROC_SERVER)?;
        dests.SetAppID(&app_id_hstring)?;
        dests.RemoveAllDestinations()?;
    }

    #[cfg(target_os = "macos")]
    unsafe {
        let mtm = MainThreadMarker::new().expect("AppKit API must be called on the main thread");
        let controller = NSDocumentController::sharedDocumentController(mtm);
        controller.clearRecentDocuments(None);
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        Err(crate::error::Error::UnsupportedPlatform)?;
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

        let app_id = &_app.config().identifier;
        let app_id_hstring = HSTRING::from(app_id);

        let doc_lists: IApplicationDocumentLists =
            CoCreateInstance(&ApplicationDocumentLists, None, CLSCTX_INPROC_SERVER)?;
        doc_lists.SetAppID(&app_id_hstring)?;
        let obj_array: IObjectArray = doc_lists.GetList(ADLT_RECENT, 30)?;
        let count = obj_array.GetCount()?;

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
        Err(crate::error::Error::UnsupportedPlatform)?;
    }

    Ok(recent_docs)
}
