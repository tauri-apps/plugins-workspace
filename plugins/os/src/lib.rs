use std::{fmt::Display, path::PathBuf};

pub use os_info::Version;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

mod commands;
mod error;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

pub struct OperatingSystem<R: Runtime>(AppHandle<R>);

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

impl<R: Runtime> OperatingSystem<R> {
    pub fn platform(&self) -> &'static str {
        match std::env::consts::OS {
            "windows" => "win32",
            "macos" => "darwin",
            _ => std::env::consts::OS,
        }
    }

    pub fn version(&self) -> Version {
        os_info::get().version().clone()
    }

    pub fn kind(&self) -> Kind {
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

    pub fn arch(&self) -> &'static str {
        std::env::consts::ARCH
    }

    pub fn tempdir(&self) -> PathBuf {
        std::env::temp_dir()
    }
}

pub trait OperatingSystemExt<R: Runtime> {
    fn os(&self) -> &OperatingSystem<R>;
}

impl<R: Runtime, T: Manager<R>> OperatingSystemExt<R> for T {
    fn os(&self) -> &OperatingSystem<R> {
        self.state::<OperatingSystem<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("os")
        .invoke_handler(tauri::generate_handler![
            commands::platform,
            commands::version,
            commands::kind,
            commands::arch,
            commands::tempdir
        ])
        .setup(|app, _api| {
            app.manage(OperatingSystem(app.clone()));
            Ok(())
        })
        .build()
}
