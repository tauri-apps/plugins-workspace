// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fmt::Display;

pub use os_info::Version;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;
mod error;

pub use error::Error;

pub enum OsType {
    Linux,
    Windows,
    Macos,
    IOS,
    Android,
}

impl Display for OsType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linux => write!(f, "linux"),
            Self::Windows => write!(f, "windows"),
            Self::Macos => write!(f, "macos"),
            Self::IOS => write!(f, "ios"),
            Self::Android => write!(f, "android"),
        }
    }
}

/// Returns a string describing the specific operating system in use, see [std::env::consts::OS].
pub fn platform() -> &'static str {
    std::env::consts::OS
}

/// Returns the current operating system version.
pub fn version() -> Version {
    os_info::get().version().clone()
}

/// Returns the current operating system type.
pub fn type_() -> OsType {
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    return OsType::Linux;
    #[cfg(target_os = "windows")]
    return OsType::Windows;
    #[cfg(target_os = "macos")]
    return OsType::Macos;
    #[cfg(target_os = "ios")]
    return OsType::IOS;
    #[cfg(target_os = "android")]
    return OsType::Android;
}

/// Returns the current operating system family, see [std::env::consts::FAMILY].
pub fn family() -> &'static str {
    std::env::consts::FAMILY
}

/// Returns the current operating system architecture, see [std::env::consts::ARCH].
pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

/// Returns the file extension, if any, used for executable binaries on this platform. Example value is `exe`, see [std::env::consts::EXE_EXTENSION].
pub fn exe_extension() -> &'static str {
    std::env::consts::EXE_EXTENSION
}

/// Returns the current operating system locale with the `BCP-47` language tag. If the locale couldn’t be obtained, `None` is returned instead.
pub fn locale() -> Option<String> {
    sys_locale::get_locale()
}

/// Returns the current operating system hostname.
pub fn hostname() -> String {
    gethostname::gethostname().to_string_lossy().to_string()
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let mut init_script = String::new();
    init_script.push_str(include_str!("api-iife.js"));
    #[cfg(windows)]
    let eol = "\r\n";
    #[cfg(not(windows))]
    let eol = "\n";
    init_script.push_str(&format!("window.__TAURI_OS__.EOL = '{eol}';"));

    Builder::new("os")
        .js_init_script(init_script)
        .invoke_handler(tauri::generate_handler![
            commands::platform,
            commands::version,
            commands::os_type,
            commands::family,
            commands::arch,
            commands::exe_extension,
            commands::locale,
            commands::hostname
        ])
        .build()
}
