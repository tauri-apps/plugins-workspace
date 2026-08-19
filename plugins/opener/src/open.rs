// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to shell.

use std::{ffi::OsStr, path::Path};

pub(crate) fn open<P: AsRef<OsStr>, S: AsRef<str>>(path: P, with: Option<S>) -> crate::Result<()> {
    match with {
        Some(program) => ::open::with_detached(path, program.as_ref()),
        // `open::that_detached()` uses a detached process which can leave a short-lived
        // zombie ("Z") child process on macOS. On other platforms we keep the previous
        // detached behavior to avoid re-introducing tauri#6849 semantics.
        #[cfg(target_os = "macos")]
        None => ::open::that(path),
        #[cfg(not(target_os = "macos"))]
        None => ::open::that_detached(path),
    }
    .map_err(Into::into)
}

/// Opens URL with the program specified in `with`, or system default if `None`.
///
/// ## Platform-specific:
///
/// - **Android / iOS**: Always opens using default program.
///
/// # Examples
///
/// ```rust,ignore
/// tauri::Builder::default()
///   .setup(|app| {
///     // open the given URL on the system default browser
///     tauri_plugin_opener::open_url("https://github.com/tauri-apps/tauri", None::<&str>)?;
///     Ok(())
///   });
/// ```
pub fn open_url<P: AsRef<str>, S: AsRef<str>>(url: P, with: Option<S>) -> crate::Result<()> {
    let url = url.as_ref();
    open(url, with)
}

/// Opens path with the program specified in `with`, or system default if `None`.
///
/// ## Platform-specific:
///
/// - **Android / iOS**: Always opens using default program.
///
/// # Examples
///
/// ```rust,ignore
/// tauri::Builder::default()
///   .setup(|app| {
///     // open the given URL on the system default explorer
///     tauri_plugin_opener::open_path("/path/to/file", None::<&str>)?;
///     Ok(())
///   });
/// ```
pub fn open_path<P: AsRef<Path>, S: AsRef<str>>(path: P, with: Option<S>) -> crate::Result<()> {
    let path = path.as_ref();
    if with.is_none() {
        // Returns an IO error if not exists, and besides `exists()` is a shorthand for `metadata()`
        _ = path.metadata()?;
    }
    open(path, with)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command as SysCommand;
    use std::thread;
    use std::time::Duration;

    #[cfg(target_os = "macos")]
    fn zombie_children_count() -> usize {
        let ppid = std::process::id();
        let ppid_str = ppid.to_string();
        let output = SysCommand::new("ps")
            .args(["-axo", "pid,ppid,stat"])
            .output()
            .expect("ps must be available");

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Header line: PID PPID STAT ...
        stdout
            .lines()
            .skip(1)
            .filter_map(|line| {
                let mut it = line.split_whitespace();
                let _pid = it.next()?;
                let child_ppid = it.next()?;
                let stat = it.next()?;
                Some((child_ppid, stat))
            })
            .filter(|(child_ppid, stat)| {
                *child_ppid == ppid_str.as_str() && stat.starts_with('Z')
            })
            .count()
    }

    #[test]
    #[cfg(target_os = "macos")]
    fn open_url_does_not_leave_zombies() {
        let before = zombie_children_count();

        // We intentionally ignore errors: the important part is whether we leave behind zombies.
        for _ in 0..5 {
            let _ = open_url("https://example.com", None::<&str>);
        }

        thread::sleep(Duration::from_millis(300));
        let after = zombie_children_count();

        assert_eq!(
            before, after,
            "open_url must not leave zombie children behind (before={before}, after={after})"
        );
    }
}
