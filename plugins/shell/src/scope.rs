// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::sync::Arc;

#[allow(deprecated)]
use crate::open::Program;
use crate::process::Command;

use regex::Regex;
use tauri::ipc::ScopeObject;
use tauri::Manager;

/// Allowed representation of `Execute` command arguments.
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(untagged, deny_unknown_fields)]
#[non_exhaustive]
pub enum ExecuteArgs {
    /// No arguments
    None,

    /// A single string argument
    Single(String),

    /// Multiple string arguments
    List(Vec<String>),
}

impl ExecuteArgs {
    /// Whether the argument list is empty or not.
    pub fn is_empty(&self) -> bool {
        match self {
            Self::None => true,
            Self::Single(s) if s.is_empty() => true,
            Self::List(l) => l.is_empty(),
            _ => false,
        }
    }
}

impl From<()> for ExecuteArgs {
    fn from(_: ()) -> Self {
        Self::None
    }
}

impl From<String> for ExecuteArgs {
    fn from(string: String) -> Self {
        Self::Single(string)
    }
}

impl From<Vec<String>> for ExecuteArgs {
    fn from(vec: Vec<String>) -> Self {
        Self::List(vec)
    }
}

/// A configured scoped shell command.
#[derive(Debug, Clone)]
pub struct ScopeAllowedCommand {
    /// Name of the command (key).
    pub name: String,

    /// The shell command to be called.
    pub command: std::path::PathBuf,

    /// The arguments the command is allowed to be called with.
    pub args: Option<Vec<ScopeAllowedArg>>,

    /// If this command is a sidecar command.
    pub sidecar: bool,
}

impl ScopeObject for ScopeAllowedCommand {
    type Error = crate::Error;
    fn deserialize<R: tauri::Runtime>(
        app: &tauri::AppHandle<R>,
        raw: tauri::utils::acl::Value,
    ) -> Result<Self, Self::Error> {
        let scope = serde_json::from_value::<crate::scope_entry::Entry>(raw.into())?;

        let args = match scope.args.clone() {
            crate::scope_entry::ShellAllowedArgs::Flag(true) => None,
            crate::scope_entry::ShellAllowedArgs::Flag(false) => Some(Vec::new()),
            crate::scope_entry::ShellAllowedArgs::List(list) => {
                let list = list.into_iter().map(|arg| match arg {
                    crate::scope_entry::ShellAllowedArg::Fixed(fixed) => {
                        crate::scope::ScopeAllowedArg::Fixed(fixed)
                    }
                    crate::scope_entry::ShellAllowedArg::Var { validator, raw } => {
                        let regex = if raw {
                            validator
                        } else {
                            format!("^{validator}$")
                        };
                        let validator = Regex::new(&regex)
                            .unwrap_or_else(|e| panic!("invalid regex {regex}: {e}"));
                        crate::scope::ScopeAllowedArg::Var { validator }
                    }
                });
                Some(list.collect())
            }
        };

        let command = if let Ok(path) = app.path().parse(&scope.command) {
            path
        } else {
            scope.command.clone()
        };

        Ok(Self {
            name: scope.name,
            command,
            args,
            sidecar: scope.sidecar,
        })
    }
}

/// A configured argument to a scoped shell command.
#[derive(Debug, Clone)]
pub enum ScopeAllowedArg {
    /// A non-configurable argument.
    Fixed(String),

    /// An argument with a value to be evaluated at runtime, must pass a regex validation.
    Var {
        /// The validation that the variable value must pass in order to be called.
        validator: Regex,
    },
}

impl ScopeAllowedArg {
    /// If the argument is fixed.
    pub fn is_fixed(&self) -> bool {
        matches!(self, Self::Fixed(_))
    }
}

/// Scope for the open command
pub struct OpenScope {
    /// The validation regex that `shell > open` paths must match against.
    pub open: Option<Regex>,
}

/// Scope for shell process spawning.
#[derive(Clone)]
pub struct ShellScope<'a> {
    /// All allowed commands, using their unique command name as the keys.
    pub scopes: Vec<&'a Arc<ScopeAllowedCommand>>,
}

/// All errors that can happen while validating a scoped command.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// At least one argument did not pass input validation.
    #[error("The scoped command was called with the improper sidecar flag set")]
    BadSidecarFlag,

    /// The sidecar program validated but failed to find the sidecar path.
    #[error(
    "The scoped sidecar command was validated, but failed to create the path to the command: {0}"
  )]
    Sidecar(String),

    /// The named command was not found in the scoped config.
    #[error("Scoped command {0} not found")]
    NotFound(String),

    /// A command variable has no value set in the arguments.
    #[error(
    "Scoped command argument at position {0} must match regex validation {1} but it was not found"
  )]
    MissingVar(usize, String),

    /// At least one argument did not pass input validation.
    #[error("Scoped command argument at position {index} was found, but failed regex validation {validation}")]
    Validation {
        /// Index of the variable.
        index: usize,

        /// Regex that the variable value failed to match.
        validation: String,
    },

    /// The format of the passed input does not match the expected shape.
    ///
    /// This can happen from passing a string or array of strings to a command that is expecting
    /// named variables, and vice-versa.
    #[error("Scoped command {0} received arguments in an unexpected format")]
    InvalidInput(String),

    /// A generic IO error that occurs while executing specified shell commands.
    #[error("Scoped shell IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl OpenScope {
    /// Open a path in the default (or specified) browser.
    ///
    /// The path is validated against the `plugins > shell > open` validation regex, which
    /// defaults to `^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+`.
    #[allow(deprecated)]
    pub fn open(&self, path: &str, with: Option<Program>) -> Result<(), Error> {
        // ensure we pass validation if the configuration has one
        if let Some(regex) = &self.open {
            if !regex.is_match(path) {
                return Err(Error::Validation {
                    index: 0,
                    validation: regex.as_str().into(),
                });
            }
        }

        // The prevention of argument escaping is handled by the usage of std::process::Command::arg by
        // the `open` dependency. This behavior should be re-confirmed during upgrades of `open`.
        match with.map(Program::name) {
            Some(program) => ::open::with_detached(path, program),
            None => ::open::that_detached(path),
        }
        .map_err(Into::into)
    }
}

impl ShellScope<'_> {
    /// Validates argument inputs and creates a Tauri sidecar [`Command`].
    pub fn prepare_sidecar(
        &self,
        command_name: &str,
        command_script: &str,
        args: ExecuteArgs,
    ) -> Result<Command, Error> {
        self._prepare(command_name, args, Some(command_script))
    }

    /// Validates argument inputs and creates a Tauri [`Command`].
    pub fn prepare(&self, command_name: &str, args: ExecuteArgs) -> Result<Command, Error> {
        self._prepare(command_name, args, None)
    }

    /// Validates argument inputs and creates a Tauri [`Command`].
    pub fn _prepare(
        &self,
        command_name: &str,
        args: ExecuteArgs,
        sidecar: Option<&str>,
    ) -> Result<Command, Error> {
        let command = match self.scopes.iter().find(|s| s.name == command_name) {
            Some(command) => command,
            None => return Err(Error::NotFound(command_name.into())),
        };

        if command.sidecar != sidecar.is_some() {
            return Err(Error::BadSidecarFlag);
        }

        let args = match (&command.args, args) {
            (None, ExecuteArgs::None) => Ok(vec![]),
            (None, ExecuteArgs::List(list)) => Ok(list),
            (None, ExecuteArgs::Single(string)) => Ok(vec![string]),
            (Some(list), ExecuteArgs::List(args)) => list
                .iter()
                .enumerate()
                .map(|(i, arg)| match arg {
                    ScopeAllowedArg::Fixed(fixed) => Ok(fixed.to_string()),
                    ScopeAllowedArg::Var { validator } => {
                        let value = args
                            .get(i)
                            .ok_or_else(|| Error::MissingVar(i, validator.to_string()))?
                            .to_string();
                        if validator.is_match(&value) {
                            Ok(value)
                        } else {
                            Err(Error::Validation {
                                index: i,
                                validation: validator.to_string(),
                            })
                        }
                    }
                })
                .collect(),
            (Some(list), arg) if arg.is_empty() && list.iter().all(ScopeAllowedArg::is_fixed) => {
                list.iter()
                    .map(|arg| match arg {
                        ScopeAllowedArg::Fixed(fixed) => Ok(fixed.to_string()),
                        _ => unreachable!(),
                    })
                    .collect()
            }
            (Some(list), _) if list.is_empty() => Err(Error::InvalidInput(command_name.into())),
            (Some(_), _) => Err(Error::InvalidInput(command_name.into())),
        }?;

        let command_s = sidecar
            .map(|s| {
                std::path::PathBuf::from(s)
                    .components()
                    .last()
                    .unwrap()
                    .as_os_str()
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap_or_else(|| command.command.to_string_lossy().into_owned());
        let command = if command.sidecar {
            Command::new_sidecar(command_s).map_err(|e| Error::Sidecar(e.to_string()))?
        } else {
            Command::new(command_s)
        };

        Ok(command.args(args))
    }
}
