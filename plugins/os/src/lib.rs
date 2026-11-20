// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Read information about the operating system and the device.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use std::fmt::Display;

pub use os_info::Version;
use serialize_to_javascript::{DefaultTemplate, Template, default_template};
use tauri::{
    Manager, Runtime,
    plugin::{Builder, TauriPlugin},
};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};
pub use models::*;

#[cfg(desktop)]
pub use desktop::Os;
#[cfg(mobile)]
pub use mobile::Os;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the device APIs of the os plugin.
pub trait OsExt<R: Runtime> {
    /// Returns a handle to the [`Os`] device APIs.
    fn os(&self) -> &Os<R>;
}

impl<R: Runtime, T: Manager<R>> crate::OsExt<R> for T {
    fn os(&self) -> &Os<R> {
        self.state::<Os<R>>().inner()
    }
}

/// The type of the current operating system, as returned by [`type_`].
pub enum OsType {
    /// Linux and Linux-based systems such as FreeBSD, DragonFly BSD, NetBSD and OpenBSD.
    Linux,
    /// Windows.
    Windows,
    /// macOS.
    Macos,
    /// iOS.
    IOS,
    /// Android.
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
///
/// On Android this is the user-visible release, e.g. `14.0.0` for Android 14, and on iOS the system version, e.g. `17.4.1`. Before 2.5.0 this was [`Version::Unknown`] on both.
pub fn version() -> Version {
    // os_info does not know the version on Android and iOS
    mobile_version().unwrap_or_else(|| os_info::get().version().clone())
}

#[cfg(target_os = "android")]
fn mobile_version() -> Option<Version> {
    android_system_property(c"ro.build.version.release").map(Version::from_string)
}

#[cfg(target_os = "ios")]
fn mobile_version() -> Option<Version> {
    // unlike the kernel version, this is the simulated iOS version on the simulator
    let version = objc2_foundation::NSProcessInfo::processInfo().operatingSystemVersion();
    Some(Version::Semantic(
        version.majorVersion as u64,
        version.minorVersion as u64,
        version.patchVersion as u64,
    ))
}

#[cfg(desktop)]
fn mobile_version() -> Option<Version> {
    None
}

/// Reads an Android system property, which is how `android.os.Build` gets its values.
#[cfg(target_os = "android")]
fn android_system_property(name: &std::ffi::CStr) -> Option<String> {
    let mut buf = [0 as std::ffi::c_char; libc::PROP_VALUE_MAX as usize];
    // SAFETY: `name` is NUL-terminated and `buf` is PROP_VALUE_MAX bytes long, the maximum the call writes.
    let len = unsafe { libc::__system_property_get(name.as_ptr(), buf.as_mut_ptr()) };
    if len <= 0 {
        return None;
    }
    // SAFETY: on success the value is NUL-terminated within `buf`.
    let value = unsafe { std::ffi::CStr::from_ptr(buf.as_ptr()) };
    Some(value.to_string_lossy().into_owned())
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

/// Returns the current operating system locale with the `BCP-47` language tag. If the locale couldn't be obtained, `None` is returned instead.
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

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let init_js = InitJavascript::new()
        .render_default(&Default::default())
        // this will never fail with the above global_os_api values
        .unwrap();

    Builder::new("os")
        .js_init_script(init_js.to_string())
        .invoke_handler(tauri::generate_handler![
            commands::locale,
            commands::hostname,
            commands::device_info,
            commands::device_name,
            commands::battery_info
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let os = mobile::init(app, api)?;
            #[cfg(desktop)]
            let os = desktop::init(app, api)?;
            app.manage(os);
            Ok(())
        })
        .build()
}
