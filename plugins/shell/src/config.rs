// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::{de::Error as DeError, Deserialize, Deserializer};

/// Configuration for the shell plugin.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    /// Access scope for the binary execution APIs.
    /// Sidecars are automatically enabled.
    #[serde(default)]
    pub scope: ShellAllowlistScope,
    /// Open URL with the user's default application.
    #[serde(default)]
    pub open: ShellAllowlistOpen,
}

/// A command allowed to be executed by the webview API.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ShellAllowedCommand {
    /// The name for this allowed shell command configuration.
    ///
    /// This name will be used inside of the webview API to call this command along with
    /// any specified arguments.
    pub name: String,

    /// The command name.
    /// It can start with a variable that resolves to a system base directory.
    /// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
    /// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
    /// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
    /// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
    // use default just so the schema doesn't flag it as required
    pub command: PathBuf,

    /// The allowed arguments for the command execution.
    pub args: ShellAllowedArgs,

    /// If this command is a sidecar command.
    pub sidecar: bool,
}

impl<'de> Deserialize<'de> for ShellAllowedCommand {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct InnerShellAllowedCommand {
            name: String,
            #[serde(rename = "cmd")]
            command: Option<PathBuf>,
            #[serde(default)]
            args: ShellAllowedArgs,
            #[serde(default)]
            sidecar: bool,
        }

        let config = InnerShellAllowedCommand::deserialize(deserializer)?;

        if !config.sidecar && config.command.is_none() {
            return Err(DeError::custom(
                "The shell scope `command` value is required.",
            ));
        }

        Ok(ShellAllowedCommand {
            name: config.name,
            command: config.command.unwrap_or_default(),
            args: config.args,
            sidecar: config.sidecar,
        })
    }
}

/// A set of command arguments allowed to be executed by the webview API.
///
/// A value of `true` will allow any arguments to be passed to the command. `false` will disable all
/// arguments. A list of [`ShellAllowedArg`] will set those arguments as the only valid arguments to
/// be passed to the attached command configuration.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowedArgs {
    /// Use a simple boolean to allow all or disable all arguments to this command configuration.
    Flag(bool),

    /// A specific set of [`ShellAllowedArg`] that are valid to call for the command configuration.
    List(Vec<ShellAllowedArg>),
}

impl Default for ShellAllowedArgs {
    fn default() -> Self {
        Self::Flag(false)
    }
}

/// A command argument allowed to be executed by the webview API.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowedArg {
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
        /// [regex]: https://docs.rs/regex/latest/regex/#syntax
        validator: String,
    },
}

/// Shell scope definition.
/// It is a list of command names and associated CLI arguments that restrict the API access from the webview.
#[derive(Debug, Default, PartialEq, Eq, Clone, Deserialize)]

pub struct ShellAllowlistScope(pub Vec<ShellAllowedCommand>);

/// Defines the `shell > open` api scope.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ShellAllowlistOpen {
    /// If the shell open API should be enabled.
    ///
    /// If enabled, the default validation regex (`^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`) is used.
    Flag(bool),

    /// Enable the shell open API, with a custom regex that the opened path must match against.
    ///
    /// If using a custom regex to support a non-http(s) schema, care should be used to prevent values
    /// that allow flag-like strings to pass validation. e.g. `--enable-debugging`, `-i`, `/R`.
    Validate(String),
}

impl Default for ShellAllowlistOpen {
    fn default() -> Self {
        Self::Flag(false)
    }
}
