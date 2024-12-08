// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Access the system shell. Allows you to spawn child processes and manage files and URLs using their default application.

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
use regex::Regex;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, RunEvent, Runtime,
};

mod commands;
mod config;
mod error;
#[deprecated(since = "2.1.0", note = "Use tauri-plugin-opener instead.")]
#[allow(deprecated)]
pub mod open;
pub mod process;
mod scope;
mod scope_entry;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

#[cfg(mobile)]
use tauri::plugin::PluginHandle;
#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.shell";
#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_shell);

type ChildStore = Arc<Mutex<HashMap<u32, CommandChild>>>;

pub struct Shell<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    #[cfg(mobile)]
    mobile_plugin_handle: PluginHandle<R>,
    open_scope: scope::OpenScope,
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

    /// Open a (url) path with a default or specific browser opening program.
    ///
    /// See [`crate::open::open`] for how it handles security-related measures.
    #[cfg(desktop)]
    #[deprecated(since = "2.1.0", note = "Use tauri-plugin-opener instead.")]
    #[allow(deprecated)]
    pub fn open(&self, path: impl Into<String>, with: Option<open::Program>) -> Result<()> {
        open::open(&self.open_scope, path.into(), with).map_err(Into::into)
    }

    /// Open a (url) path with a default or specific browser opening program.
    ///
    /// See [`crate::open::open`] for how it handles security-related measures.
    #[cfg(mobile)]
    #[deprecated(since = "2.1.0", note = "Use tauri-plugin-opener instead.")]
    pub fn open(&self, path: impl Into<String>, _with: Option<open::Program>) -> Result<()> {
        self.mobile_plugin_handle
            .run_mobile_plugin("open", path.into())
            .map_err(Into::into)
    }
}

pub trait ShellExt<R: Runtime> {
    fn shell(&self) -> &Shell<R>;
}

impl<R: Runtime, T: Manager<R>> ShellExt<R> for T {
    fn shell(&self) -> &Shell<R> {
        self.state::<Shell<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R, Option<config::Config>> {
    Builder::<R, Option<config::Config>>::new("shell")
        .js_init_script(include_str!("init-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![
            commands::execute,
            commands::spawn,
            commands::stdin_write,
            commands::kill,
            commands::open
        ])
        .setup(|app, api| {
            let default_config = config::Config::default();
            let config = api.config().as_ref().unwrap_or(&default_config);

            #[cfg(target_os = "android")]
            let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "ShellPlugin")?;
            #[cfg(target_os = "ios")]
            let handle = api.register_ios_plugin(init_plugin_shell)?;

            app.manage(Shell {
                app: app.clone(),
                children: Default::default(),
                open_scope: open_scope(&config.open),

                #[cfg(mobile)]
                mobile_plugin_handle: handle,
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

fn open_scope(open: &config::ShellAllowlistOpen) -> scope::OpenScope {
    let shell_scope_open = match open {
        config::ShellAllowlistOpen::Flag(false) => None,
        config::ShellAllowlistOpen::Flag(true) => {
            Some(Regex::new(r"^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+").unwrap())
        }
        config::ShellAllowlistOpen::Validate(validator) => {
            let regex = format!("^{validator}$");
            let validator =
                Regex::new(&regex).unwrap_or_else(|e| panic!("invalid regex {regex}: {e}"));
            Some(validator)
        }
    };

    scope::OpenScope {
        open: shell_scope_open,
    }
}
