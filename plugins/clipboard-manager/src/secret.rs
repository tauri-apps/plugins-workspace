// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(windows)]
use arboard::SetExtWindows;

#[cfg(target_os = "macos")]
use arboard::SetExtApple;

#[cfg(not(any(windows, target_os = "macos")))]
use arboard::SetExtLinux;

/// Hints clipboard managers to exclude the data from history.
///
/// On Windows, it calls [`arboard::SetExtWindows::exclude_from_monitoring`], which also covers history and cloud sync.
/// On macOS and Linux, it calls `exclude_from_history`.
pub(crate) fn exclude_secret(set: arboard::Set<'_>) -> arboard::Set<'_> {
    #[cfg(windows)]
    return set.exclude_from_monitoring();
    #[cfg(not(windows))]
    return set.exclude_from_history();
}
