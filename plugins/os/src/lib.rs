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

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("os")
        .js_init_script(include_str!("api-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![
            commands::platform,
            commands::version,
            commands::kind,
            commands::arch,
            commands::tempdir
        ])
        .build()
}
