// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use schemars::JsonSchema;

#[path = "src/scope_entry.rs"]
mod scope_entry;

/// A command argument allowed to be executed by the webview API.
#[derive(Debug, PartialEq, Eq, Clone, Hash, schemars::JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellScopeEntryAllowedArg {
    /// A non-configurable argument that is passed to the command in the order it was specified.
    Fixed(String),

    /// A variable that is set while calling the command from the webview API.
    ///
    Var {
        /// [regex] validator to require passed values to conform to an expected input.
        ///
        /// This will require the argument value passed to this variable to match the `validator` regex
        /// before it will be executed.
        ///
        /// The regex string is by default surrounded by `^...$` to match the full string.
        /// For example the `https?://\w+` regex would be registered as `^https?://\w+$`.
        ///
        /// [regex]: <https://docs.rs/regex/latest/regex/#syntax>
        validator: String,

        /// Marks the validator as a raw regex, meaning the plugin should not make any modification at runtime.
        ///
        /// This means the regex will not match on the entire string by default, which might
        /// be exploited if your regex allow unexpected input to be considered valid.
        /// When using this option, make sure your regex is correct.
        #[serde(default)]
        raw: bool,
    },
}

/// A set of command arguments allowed to be executed by the webview API.
///
/// A value of `true` will allow any arguments to be passed to the command. `false` will disable all
/// arguments. A list of [`ShellScopeEntryAllowedArg`] will set those arguments as the only valid arguments to
/// be passed to the attached command configuration.
#[derive(Debug, PartialEq, Eq, Clone, Hash, JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellScopeEntryAllowedArgs {
    /// Use a simple boolean to allow all or disable all arguments to this command configuration.
    Flag(bool),

    /// A specific set of [`ShellScopeEntryAllowedArg`] that are valid to call for the command configuration.
    List(Vec<ShellScopeEntryAllowedArg>),
}

impl Default for ShellScopeEntryAllowedArgs {
    fn default() -> Self {
        Self::Flag(false)
    }
}

/// Shell scope entry.
#[derive(JsonSchema)]
#[serde(untagged, deny_unknown_fields)]
#[allow(unused)]
pub(crate) enum ShellScopeEntry {
    Command {
        /// The name for this allowed shell command configuration.
        ///
        /// This name will be used inside of the webview API to call this command along with
        /// any specified arguments.
        name: String,
        /// The command name.
        /// It can start with a variable that resolves to a system base directory.
        /// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
        /// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
        /// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
        /// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
        // use default just so the schema doesn't flag it as required
        #[serde(rename = "cmd")]
        command: PathBuf,
        /// The allowed arguments for the command execution.
        #[serde(default)]
        args: ShellScopeEntryAllowedArgs,
    },
    Sidecar {
        /// The name for this allowed shell command configuration.
        ///
        /// This name will be used inside of the webview API to call this command along with
        /// any specified arguments.
        name: String,
        /// The allowed arguments for the command execution.
        #[serde(default)]
        args: ShellScopeEntryAllowedArgs,
        /// If this command is a sidecar command.
        sidecar: bool,
    },
}

// Ensure `ShellScopeEntry` and `scope_entry::EntryRaw`
// and `ShellScopeEntryAllowedArg` and `ShellAllowedArg`
// and `ShellScopeEntryAllowedArgs` and `ShellAllowedArgs`
// are kept in sync
#[allow(clippy::unnecessary_operation)]
fn _f() {
    match (ShellScopeEntry::Sidecar {
        name: String::new(),
        args: ShellScopeEntryAllowedArgs::Flag(false),
        sidecar: true,
    }) {
        ShellScopeEntry::Command {
            name,
            command,
            args,
        } => scope_entry::EntryRaw {
            name,
            command: Some(command),
            args: match args {
                ShellScopeEntryAllowedArgs::Flag(flag) => scope_entry::ShellAllowedArgs::Flag(flag),
                ShellScopeEntryAllowedArgs::List(vec) => scope_entry::ShellAllowedArgs::List(
                    vec.into_iter()
                        .map(|s| match s {
                            ShellScopeEntryAllowedArg::Fixed(fixed) => {
                                scope_entry::ShellAllowedArg::Fixed(fixed)
                            }
                            ShellScopeEntryAllowedArg::Var { validator, raw } => {
                                scope_entry::ShellAllowedArg::Var { validator, raw }
                            }
                        })
                        .collect(),
                ),
            },
            sidecar: false,
        },
        ShellScopeEntry::Sidecar {
            name,
            args,
            sidecar,
        } => scope_entry::EntryRaw {
            name,
            command: None,
            args: match args {
                ShellScopeEntryAllowedArgs::Flag(flag) => scope_entry::ShellAllowedArgs::Flag(flag),
                ShellScopeEntryAllowedArgs::List(vec) => scope_entry::ShellAllowedArgs::List(
                    vec.into_iter()
                        .map(|s| match s {
                            ShellScopeEntryAllowedArg::Fixed(fixed) => {
                                scope_entry::ShellAllowedArg::Fixed(fixed)
                            }
                            ShellScopeEntryAllowedArg::Var { validator, raw } => {
                                scope_entry::ShellAllowedArg::Var { validator, raw }
                            }
                        })
                        .collect(),
                ),
            },
            sidecar,
        },
    };
}

const COMMANDS: &[&str] = &["execute", "spawn", "stdin_write", "kill", "open"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS)
        .global_api_script_path("./api-iife.js")
        .global_scope_schema(schemars::schema_for!(ShellScopeEntry))
        .android_path("android")
        .ios_path("ios")
        .build();

    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap();
    let mobile = target_os == "ios" || target_os == "android";
    alias("desktop", !mobile);
    alias("mobile", mobile);
}

// creates a cfg alias if `has_feature` is true.
// `alias` must be a snake case string.
fn alias(alias: &str, has_feature: bool) {
    println!("cargo:rustc-check-cfg=cfg({alias})");
    if has_feature {
        println!("cargo:rustc-cfg={alias}");
    }
}
