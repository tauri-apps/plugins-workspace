use std::{ffi::OsString, path::PathBuf};

use windows::{
    core::{w, HSTRING, PCWSTR},
    Win32::{
        Foundation::{ERROR_FILE_NOT_FOUND, HWND},
        System::Com::CoInitialize,
        UI::{
            Shell::{ILCreateFromPathW, ILFree, SHOpenFolderAndSelectItems, ShellExecuteW},
            WindowsAndMessaging::SW_SHOW,
        },
    },
};

pub fn show_item_in_directory(file: PathBuf) -> crate::Result<()> {
    let _ = unsafe { CoInitialize(None) };

    let dir = file
        .parent()
        .ok_or_else(|| crate::Error::NoParent(file.clone()))?;

    let dir = OsString::from(dir);
    let dir = HSTRING::from(dir);
    let dir_item = unsafe { ILCreateFromPathW(PCWSTR::from_raw(dir.as_ptr())) };

    let file = OsString::from(file);
    let file = HSTRING::from(file);
    let file_item = unsafe { ILCreateFromPathW(PCWSTR::from_raw(file.as_ptr())) };

    unsafe {
        if let Err(e) = SHOpenFolderAndSelectItems(dir_item, Some(&[file_item]), 0) {
            if e.code().0 == ERROR_FILE_NOT_FOUND.0 as i32 {
                ShellExecuteW(
                    HWND::default(),
                    w!("open"),
                    PCWSTR::from_raw(dir.as_ptr()),
                    PCWSTR::null(),
                    PCWSTR::null(),
                    SW_SHOW,
                );
            }
        }
    }

    unsafe {
        ILFree(Some(dir_item));
        ILFree(Some(file_item));
    }

    Ok(())
}
