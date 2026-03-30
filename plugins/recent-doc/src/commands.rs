// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::error::Result;
use std::fs;
use tauri::command;

#[cfg(target_os = "windows")]
use windows::{
    core::{HSTRING, PCWSTR},
    Win32::{
        System::Com::{CoCreateInstance, CoTaskMemFree, CLSCTX_INPROC_SERVER},
        UI::Shell::{
            ApplicationDocumentLists, GetCurrentProcessExplicitAppUserModelID,
            IApplicationDocumentLists, IShellItem, IShellItemArray, SHAddToRecentDocs,
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

#[command]
pub(crate) fn add_recent_document(_path: &str) -> Result<()> {
    #[cfg(target_os = "windows")]
    unsafe {
        // Convert path to HSTRING
        let path_hstring = HSTRING::from(_path);
        let item = SHCreateItemFromParsingName(&path_hstring, None)?;

        // get app id pointer
        let app_id_pwstr = GetCurrentProcessExplicitAppUserModelID()?;

        // construct info
        let info = SHARDAPPIDINFO {
            pszAppID: PCWSTR::from_raw(app_id_pwstr.as_ptr()),
            psi: std::mem::ManuallyDrop::new(Some(item)),
        };

        SHAddToRecentDocs(
            SHARD_APPIDINFO.0 as u32,
            Some(&info as *const _ as *const core::ffi::c_void),
        );

        // Free memory
        CoTaskMemFree(Some(app_id_pwstr.as_ptr() as *mut _));
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
        // Instantiate the IApplicationDocumentLists COM object
        // It automatically scopes to the calling process's AppUserModelID
        let app_docs: IApplicationDocumentLists = CoCreateInstance(
            &ApplicationDocumentLists as *const _,
            None,
            CLSCTX_INPROC_SERVER,
        )?;

        // Retrieve the recent document list specific to this application
        let obj_array: IShellItemArray = app_docs.GetList(ADLT_RECENT, 0)?;
        let count = obj_array.GetCount()?;

        // Iterate through the returned collection
        for i in 0..count {
            // Extract the generic ShellItem
            let item: IShellItem = obj_array.GetItemAt(i)?;

            // Extract the absolute file system path
            let name_pwstr = item.GetDisplayName(SIGDN_FILESYSPATH)?;
            if !name_pwstr.as_ptr().is_null() {
                recent_docs.push(name_pwstr.to_string()?);
                // Free the memory allocated by the COM subsystem
                CoTaskMemFree(Some(name_pwstr.0 as *mut _));
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

    #[cfg(unix)]
    {
        // Recent documents are not supported on Unix-like systems.
        Err(crate::error::Error::UnsupportedPlatform(
            "Recent documents are not supported on Unix-like systems.".to_string(),
        ))?;
    }

    Ok(recent_docs)
}
