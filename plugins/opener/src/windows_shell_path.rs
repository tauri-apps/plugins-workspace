// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    borrow::Cow,
    ffi::OsString,
    io,
    os::windows::ffi::OsStringExt,
    path::{Component, Path, PathBuf, Prefix, PrefixComponent},
};

use windows::{core::HSTRING, Win32::Storage::FileSystem::GetFullPathNameW};

pub fn absolute_and_check_exists(path: &Path) -> io::Result<PathBuf> {
    let path = absolute(path)?;
    if path.exists() {
        Ok(path)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "path doesn't exist",
        ))
    }
}

// TODO: Switch to use `std::path::absolute` once MSRV > 1.79
// Modified from https://github.com/rust-lang/rust/blob/b49ecc9eb70a51e89f32a7358e790f7b3808ccb3/library/std/src/sys/path/windows.rs#L185
// Note: this doesn't resolve symlinks
pub fn absolute(path: &Path) -> io::Result<PathBuf> {
    if path.as_os_str().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "cannot make an empty path absolute",
        ));
    }

    let prefix = path.components().next();
    // Verbatim paths should not be modified.
    if prefix
        .map(|component| {
            let Component::Prefix(prefix) = component else {
                return false;
            };
            matches!(
                prefix.kind(),
                Prefix::Verbatim(..) | Prefix::VerbatimDisk(..) | Prefix::VerbatimUNC(..)
            )
        })
        .unwrap_or(false)
    {
        // NULs in verbatim paths are rejected for consistency.
        if path.as_os_str().as_encoded_bytes().contains(&0) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "strings passed to WinAPI cannot contain NULs",
            ));
        }
        return Ok(path.to_owned());
    }

    // This is an additional check to make sure we don't pass in a single driver letter to GetFullPathNameW
    // which will resolves to the current working directory
    //
    // > https://learn.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-getfullpathnamew#:~:text=If%20you%20specify%20%22U%3A%22%20the%20path%20returned%20is%20the%20current%20directory%20on%20the%20%22U%3A%5C%22%20drive
    #[allow(clippy::collapsible_if)]
    if let Some(Component::Prefix(last_prefix)) = path.components().next_back() {
        if matches!(last_prefix.kind(), Prefix::Disk(..)) {
            return Ok(PathBuf::from(last_prefix.as_os_str()));
        }
    }

    let path_hstring = HSTRING::from(path);

    let size = unsafe { GetFullPathNameW(&path_hstring, None, None) };
    if size == 0 {
        return Err(io::Error::last_os_error());
    }
    let mut buffer = vec![0; size as usize];
    let size = unsafe { GetFullPathNameW(&path_hstring, Some(&mut buffer), None) };
    if size == 0 {
        return Err(io::Error::last_os_error());
    }

    Ok(PathBuf::from(OsString::from_wide(&buffer[..size as usize])))
}

/// Similar to [`Path::parent`] but resolves parent of `C:`/`C:\` to `""` and handles UNC host name (`\\wsl.localhost\Ubuntu\` to `\\wsl.localhost`)
pub fn shell_parent_path(path: &Path) -> Option<Cow<'_, Path>> {
    fn handle_prefix(prefix: PrefixComponent<'_>) -> Option<Cow<'_, Path>> {
        match prefix.kind() {
            Prefix::UNC(host_name, _share_name) => {
                let mut path = OsString::from(r"\\");
                path.push(host_name);
                Some(PathBuf::from(path).into())
            }
            Prefix::Disk(_) => Some(PathBuf::from("").into()),
            _ => None,
        }
    }

    let mut components = path.components();
    let component = components.next_back()?;
    match component {
        Component::Normal(_) | Component::CurDir | Component::ParentDir => {
            Some(components.as_path().into())
        }
        Component::Prefix(prefix) => handle_prefix(prefix),
        // Handle cases like `C:\` and `\\wsl.localhost\Ubuntu\`
        Component::RootDir => {
            if let Component::Prefix(prefix) = components.next_back()? {
                handle_prefix(prefix)
            } else {
                None
            }
        }
    }
}
