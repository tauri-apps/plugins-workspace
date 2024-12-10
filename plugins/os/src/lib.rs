// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Read information about the operating system.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use std::fmt::Display;

pub use os_info::Version;
use serialize_to_javascript::{default_template, DefaultTemplate, Template};
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

#[derive(Template)]
#[default_template("./init.js")]
struct InitJavascript<'a> {
    eol: &'static str,
    os_type: String,
    platform: &'a str,
    family: &'a str,
    version: String,
    arch: &'a str,
    exe_extension: &'a str,
}

impl InitJavascript<'_> {
    fn new() -> Self {
        Self {
            #[cfg(windows)]
            eol: "\r\n",
            #[cfg(not(windows))]
            eol: "\n",
            os_type: crate::type_().to_string(),
            platform: crate::platform(),
            family: crate::family(),
            version: crate::version().to_string(),
            arch: crate::arch(),
            exe_extension: crate::exe_extension(),
        }
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let init_js = InitJavascript::new()
        .render_default(&Default::default())
        // this will never fail with the above global_os_api values
        .unwrap();

    Builder::new("os")
        .js_init_script(init_js.to_string())
        .invoke_handler(tauri::generate_handler![
            commands::locale,
            commands::hostname
        ])
        .build()
}
