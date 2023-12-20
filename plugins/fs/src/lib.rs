// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/fs/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/fs)
//!
//! Access the file system.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    scope::fs::Scope,
    utils::config::FsScope,
    FileDropEvent, Manager, RunEvent, Runtime, WindowEvent,
};

mod commands;
mod config;
mod error;
#[cfg(feature = "watch")]
mod watcher;

pub use config::Config;
pub use error::Error;

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
        .js_init_script(include_str!("api-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![
            commands::create,
            commands::open,
            commands::copy_file,
            commands::close,
            commands::mkdir,
            commands::read_dir,
            commands::read,
            commands::read_file,
            commands::read_text_file,
            commands::read_text_file_lines,
            commands::read_text_file_lines_next,
            commands::remove,
            commands::rename,
            commands::seek,
            commands::stat,
            commands::lstat,
            commands::fstat,
            commands::truncate,
            commands::ftruncate,
            commands::write,
            commands::write_file,
            commands::write_text_file,
            commands::exists,
            #[cfg(feature = "watch")]
            watcher::watch,
            #[cfg(feature = "watch")]
            watcher::unwatch
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
                event: WindowEvent::FileDrop(FileDropEvent::Dropped { paths, position: _ }),
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
