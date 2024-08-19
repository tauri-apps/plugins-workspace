// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{de::Error as DeError, Deserialize, Deserializer};

use std::path::PathBuf;

/// A command allowed to be executed by the webview API.
#[derive(Debug, Clone, PartialEq, Eq, Hash, schemars::JsonSchema)]
pub struct Entry {
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
    #[serde(rename = "cmd")]
    pub command: PathBuf,

    /// The allowed arguments for the command execution.
    pub args: ShellAllowedArgs,

    /// If this command is a sidecar command.
    pub sidecar: bool,
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct InnerEntry {
            name: String,
            #[serde(rename = "cmd")]
            command: Option<PathBuf>,
            #[serde(default)]
            args: ShellAllowedArgs,
            #[serde(default)]
            sidecar: bool,
        }

        let config = InnerEntry::deserialize(deserializer)?;

        if !config.sidecar && config.command.is_none() {
            return Err(DeError::custom(
                "The shell scope `command` value is required.",
            ));
        }

        Ok(Entry {
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
#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize, schemars::JsonSchema)]
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
#[derive(Debug, PartialEq, Eq, Clone, Hash, Deserialize, schemars::JsonSchema)]
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
