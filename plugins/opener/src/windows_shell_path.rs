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
fn absolute(path: &Path) -> io::Result<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    // absolute() tests

    #[test]
    fn absolute_empty_error() {
        let err = absolute(Path::new("")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn absolute_verbatim_passthrough() {
        let path = Path::new(r"\\?\C:\foo");
        assert_eq!(absolute(path).unwrap(), path);
    }

    #[test]
    fn absolute_verbatim_unc_passthrough() {
        let path = Path::new(r"\\?\UNC\server\share");
        assert_eq!(absolute(path).unwrap(), path);
    }

    #[test]
    fn absolute_bare_drive_letter() {
        let result = absolute(Path::new("C:")).unwrap();
        assert_eq!(result, Path::new("C:"));
    }

    #[test]
    fn absolute_already_absolute() {
        let result = absolute(Path::new(r"C:\Windows")).unwrap();
        assert_eq!(result, Path::new(r"C:\Windows"));
    }

    #[test]
    fn absolute_unc_path() {
        let result = absolute(Path::new(r"\\server\share\folder")).unwrap();
        assert_eq!(result, Path::new(r"\\server\share\folder"));
    }

    #[test]
    fn absolute_converts_forward_slashes() {
        let result = absolute(Path::new("C:/Windows/System32")).unwrap();
        assert_eq!(result, Path::new(r"C:\Windows\System32"));
    }

    // absolute_and_check_exists() tests

    #[test]
    fn absolute_and_check_exists_existing_path() {
        assert!(absolute_and_check_exists(Path::new(r"C:\Windows")).is_ok());
    }

    #[test]
    fn absolute_and_check_exists_nonexistent_path() {
        let err = absolute_and_check_exists(Path::new(r"C:\nonexistent_xyz_12345")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::NotFound);
    }

    #[test]
    fn absolute_and_check_exists_empty_propagates() {
        let err = absolute_and_check_exists(Path::new("")).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    // shell_parent_path() tests

    #[test]
    fn shell_parent_path_local_path() {
        let result = shell_parent_path(Path::new(r"C:\Users\foo"));
        assert_eq!(result.as_deref(), Some(Path::new(r"C:\Users")));
    }

    #[test]
    fn shell_parent_path_nested_path() {
        let result = shell_parent_path(Path::new(r"C:\a\b\c\d"));
        assert_eq!(result.as_deref(), Some(Path::new(r"C:\a\b\c")));
    }

    #[test]
    fn shell_parent_path_drive_root_trailing() {
        let result = shell_parent_path(Path::new(r"C:\"));
        assert_eq!(result.as_deref(), Some(Path::new("")));
    }

    #[test]
    fn shell_parent_path_bare_drive() {
        let result = shell_parent_path(Path::new("C:"));
        assert_eq!(result.as_deref(), Some(Path::new("")));
    }

    #[test]
    fn shell_parent_path_unc_with_subfolder() {
        let result = shell_parent_path(Path::new(r"\\server\share\folder"));
        assert_eq!(result.as_deref(), Some(Path::new(r"\\server\share")));
    }

    #[test]
    fn shell_parent_path_unc_share_trailing_slash() {
        let result = shell_parent_path(Path::new(r"\\server.local\share\"));
        assert_eq!(result.as_deref(), Some(Path::new(r"\\server.local")));
    }

    #[test]
    fn shell_parent_path_unc_share_no_slash() {
        let result = shell_parent_path(Path::new(r"\\server\share"));
        assert_eq!(result.as_deref(), Some(Path::new(r"\\server")));
    }

    #[test]
    fn shell_parent_path_relative() {
        let result = shell_parent_path(Path::new(r"foo\bar"));
        assert_eq!(result.as_deref(), Some(Path::new("foo")));
    }

    #[test]
    fn shell_parent_path_single_component() {
        let result = shell_parent_path(Path::new("foo"));
        assert_eq!(result.as_deref(), Some(Path::new("")));
    }

    #[test]
    fn shell_parent_path_empty() {
        let result = shell_parent_path(Path::new(""));
        assert!(result.is_none());
    }

    #[test]
    fn shell_parent_path_verbatim() {
        let result = shell_parent_path(Path::new(r"\\?\C:\foo"));
        assert_eq!(result.as_deref(), Some(Path::new(r"\\?\C:\")));
    }
}
