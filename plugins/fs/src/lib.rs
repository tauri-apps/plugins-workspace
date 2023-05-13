// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use config::FsScope;
use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    FileDropEvent, Manager, RunEvent, Runtime, WindowEvent,
};

mod commands;
mod config;
mod error;
mod scope;

pub use config::Config;
pub use error::Error;
pub use scope::{Event as ScopeEvent, Scope};

type Result<T> = std::result::Result<T, Error>;

pub trait FsExt<R: Runtime> {
    fn fs_scope(&self) -> &Scope;
    fn try_fs_scope(&self) -> Option<&Scope>;
}

impl<R: Runtime, T: Manager<R>> FsExt<R> for T {
    fn fs_scope(&self) -> &Scope {
        self.state::<Scope>().inner()
    }

    fn try_fs_scope(&self) -> Option<&Scope> {
        self.try_state::<Scope>().map(|s| s.inner())
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R, Option<Config>> {
    PluginBuilder::<R, Option<Config>>::new("fs")
        .invoke_handler(tauri::generate_handler![
            #[cfg(feature = "allow-read-file")]
            commands::read_file,
            #[cfg(feature = "allow-read-file")]
            commands::read_text_file,
            #[cfg(feature = "allow-write-file")]
            commands::write_file,
            #[cfg(feature = "allow-read-dir")]
            commands::read_dir,
            #[cfg(feature = "allow-copy-file")]
            commands::copy_file,
            #[cfg(feature = "allow-create-dir")]
            commands::create_dir,
            #[cfg(feature = "allow-remove-dir")]
            commands::remove_dir,
            #[cfg(feature = "allow-remove-file")]
            commands::remove_file,
            #[cfg(feature = "allow-rename-file")]
            commands::rename_file,
            #[cfg(feature = "allow-exists")]
            commands::exists,
            #[cfg(feature = "allow-metadata")]
            commands::metadata
        ])
        .setup(|app: &tauri::AppHandle<R>, api| {
            let default_scope = FsScope::default();
            app.manage(Scope::new(
                app,
                api.config()
                    .as_ref()
                    .map(|c| &c.scope)
                    .unwrap_or(&default_scope),
            )?);
            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::WindowEvent {
                label: _,
                event: WindowEvent::FileDrop(FileDropEvent::Dropped(paths)),
                ..
            } = event
            {
                let scope = app.fs_scope();
                for path in paths {
                    if path.is_file() {
                        let _ = scope.allow_file(path);
                    } else {
                        let _ = scope.allow_directory(path, false);
                    }
                }
            }
        })
        .build()
}
