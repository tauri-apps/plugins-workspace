// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Parse arguments from your Command Line Interface.
//!
//! - Supported platforms: Windows, Linux and macOS.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use tauri::{
    plugin::{Builder, PluginApi, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};

mod config;
mod error;
mod parser;

use config::{Arg, Config};

pub use error::{Error, Result};
pub use parser::{ArgData, Matches, SubcommandMatches};

/// Access to the CLI APIs, managed by the app once the plugin is initialized.
pub struct Cli<R: Runtime>(PluginApi<R, Config>);

impl<R: Runtime> Cli<R> {
    /// Parses the arguments the current process was started with against the CLI definition
    /// configured under `plugins.cli` in `tauri.conf.json` and returns the resolved
    /// [`parser::Matches`]. Errors if the arguments do not satisfy the CLI definition.
    pub fn matches(&self) -> Result<parser::Matches> {
        parser::get_matches(self.0.config(), self.0.app().package_info(), None)
    }

    /// Same as [`Self::matches`], but parses the given `args` instead of the current process'
    /// arguments. Errors if `args` does not satisfy the CLI definition.
    pub fn matches_from(&self, args: Vec<String>) -> Result<parser::Matches> {
        parser::get_matches(self.0.config(), self.0.app().package_info(), Some(args))
    }
}

/// Extension trait to access the CLI APIs.
pub trait CliExt<R: Runtime> {
    /// Returns the [`Cli`] instance managed by the app.
    fn cli(&self) -> &Cli<R>;
}

impl<R: Runtime, T: Manager<R>> CliExt<R> for T {
    fn cli(&self) -> &Cli<R> {
        self.state::<Cli<R>>().inner()
    }
}

#[tauri::command]
fn cli_matches<R: Runtime>(_app: AppHandle<R>, cli: State<'_, Cli<R>>) -> Result<parser::Matches> {
    cli.matches()
}

/// Initializes the plugin, reading the CLI definition from the `plugins.cli` object in
/// `tauri.conf.json`.
pub fn init<R: Runtime>() -> TauriPlugin<R, Config> {
    Builder::new("cli")
        .invoke_handler(tauri::generate_handler![cli_matches])
        .setup(|app, api| {
            app.manage(Cli(api));
            Ok(())
        })
        .build()
}
