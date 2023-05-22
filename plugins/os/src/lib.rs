// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt::Display, path::PathBuf};

pub use os_info::Version;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;
mod error;

pub use error::Error;

pub enum Kind {
    Linux,
    Windows,
    Darwin,
    IOS,
    Android,
}

impl Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Linux => write!(f, "Linux"),
            Self::Windows => write!(f, "Linux_NT"),
            Self::Darwin => write!(f, "Darwin"),
            Self::IOS => write!(f, "iOS"),
            Self::Android => write!(f, "Android"),
        }
    }
}

pub fn platform() -> &'static str {
    match std::env::consts::OS {
        "windows" => "win32",
        "macos" => "darwin",
        _ => std::env::consts::OS,
    }
}

pub fn version() -> Version {
    os_info::get().version().clone()
}

pub fn kind() -> Kind {
    #[cfg(target_os = "linux")]
    return Kind::Linux;
    #[cfg(target_os = "windows")]
    return Kind::Windows;
    #[cfg(target_os = "macos")]
    return Kind::Darwin;
    #[cfg(target_os = "ios")]
    return Kind::IOS;
    #[cfg(target_os = "android")]
    return Kind::Android;
}

pub fn arch() -> &'static str {
    std::env::consts::ARCH
}

pub fn tempdir() -> PathBuf {
    std::env::temp_dir()
}

/// Returns the locale with the `BCP-47` language tag. If the locale couldn’t be obtained, `None` is returned instead.
pub fn locale() -> Option<String> {
    sys_locale::get_locale()
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("os")
        .invoke_handler(tauri::generate_handler![
            commands::platform,
            commands::version,
            commands::kind,
            commands::arch,
            commands::tempdir,
            commands::locale
        ])
        .build()
}
