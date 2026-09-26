// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Access the system shell. Allows you to spawn child processes.
//!
//! To open files and URLs with their default application, use `tauri-plugin-opener`.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use std::{
    collections::HashMap,
    ffi::OsStr,
    path::Path,
    sync::{Arc, Mutex},
};

use process::{Command, CommandChild};
use tauri::{
    AppHandle, Manager, RunEvent, Runtime,
    plugin::{Builder, TauriPlugin},
};

mod commands;
mod error;
pub mod process;
mod scope;
mod scope_entry;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

type ChildStore = Arc<Mutex<HashMap<u32, CommandChild>>>;

/// Access to the shell APIs.
///
/// Get an instance of this type with [`ShellExt::shell`].
pub struct Shell<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    children: ChildStore,
}

impl<R: Runtime> Shell<R> {
    /// Creates a new Command for launching the given program.
    pub fn command(&self, program: impl AsRef<OsStr>) -> Command {
        Command::new(program)
    }

    /// Creates a new Command for launching the given sidecar program.
    ///
    /// A sidecar program is a embedded external binary in order to make your application work
    /// or to prevent users having to install additional dependencies (e.g. Node.js, Python, etc).
    pub fn sidecar(&self, program: impl AsRef<Path>) -> Result<Command> {
        Command::new_sidecar(program)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`],
/// [`tauri::Webview`] and [`tauri::Window`] to access the shell APIs.
pub trait ShellExt<R: Runtime> {
    /// Gets the shell APIs.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_shell::ShellExt;
    ///
    /// async fn run_echo<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    ///     let output = app.shell().command("echo").args(["hello"]).output().await.unwrap();
    ///     println!("{}", String::from_utf8_lossy(&output.stdout));
    /// }
    /// ```
    fn shell(&self) -> &Shell<R>;
}

impl<R: Runtime, T: Manager<R>> ShellExt<R> for T {
    fn shell(&self) -> &Shell<R> {
        self.state::<Shell<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("shell")
        .initialization_script(include_str!("init-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![
            commands::execute,
            commands::spawn,
            commands::stdin_write,
            commands::kill,
        ])
        .setup(|app, _api| {
            app.manage(Shell {
                app: app.clone(),
                children: Default::default(),
            });
            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::Exit = event {
                let shell = app.state::<Shell<R>>();
                let children = {
                    let mut lock = shell.children.lock().unwrap();
                    std::mem::take(&mut *lock)
                };
                for child in children.into_values() {
                    let _ = child.kill();
                }
            }
        })
        .build()
}
