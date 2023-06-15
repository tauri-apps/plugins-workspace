// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::{
    builder::{PossibleValue, PossibleValuesParser},
    error::ErrorKind,
    Arg as ClapArg, ArgAction, ArgMatches, Command,
};
use serde::Serialize;
use serde_json::Value;
use tauri::PackageInfo;

use crate::{Arg, Config};

use std::collections::HashMap;

#[macro_use]
mod macros;

/// The resolution of a argument match.
#[derive(Default, Debug, Serialize)]
#[non_exhaustive]
pub struct ArgData {
    /// - [`Value::Bool`] if it's a flag,
    /// - [`Value::Array`] if it's multiple,
    /// - [`Value::String`] if it has value,
    /// - [`Value::Null`] otherwise.
    pub value: Value,
    /// The number of occurrences of the argument.
    /// e.g. `./app --arg 1 --arg 2 --arg 2 3 4` results in three occurrences.
    pub occurrences: u8,
}

/// The matched subcommand.
#[derive(Default, Debug, Serialize)]
#[non_exhaustive]
pub struct SubcommandMatches {
    /// The subcommand name.
    pub name: String,
    /// The subcommand argument matches.
    pub matches: Matches,
}

/// The argument matches of a command.
#[derive(Default, Debug, Serialize)]
#[non_exhaustive]
pub struct Matches {
    /// Data structure mapping each found arg with its resolution.
    pub args: HashMap<String, ArgData>,
    /// The matched subcommand if found.
    pub subcommand: Option<Box<SubcommandMatches>>,
}

impl Matches {
    /// Set a arg match.
    pub(crate) fn set_arg(&mut self, name: String, value: ArgData) {
        self.args.insert(name, value);
    }

    /// Sets the subcommand matches.
    pub(crate) fn set_subcommand(&mut self, name: String, matches: Matches) {
        self.subcommand = Some(Box::new(SubcommandMatches { name, matches }));
    }
}

/// Gets the argument matches of the CLI definition.
///
/// This is a low level API. If the application has been built,
/// prefer [`App::get_cli_matches`](`crate::App#method.get_cli_matches`).
///
/// # Examples
///
/// ```rust,no_run
/// use tauri_plugin_cli::CliExt;
/// tauri::Builder::default()
///   .setup(|app| {
///     let matches = app.cli().matches()?;
///     Ok(())
///   });
/// ```
pub fn get_matches(cli: &Config, package_info: &PackageInfo) -> crate::Result<Matches> {
    let about = cli
        .description()
        .unwrap_or(&package_info.description.to_string())
        .to_string();
    let version = package_info.version.to_string();
    let app = get_app(
        package_info,
        version,
        package_info.name.clone(),
        Some(&about),
        cli,
    );
    match app.try_get_matches() {
        Ok(matches) => Ok(get_matches_internal(cli, &matches)),
        Err(e) => match e.kind() {
            ErrorKind::DisplayHelp => {
                let mut matches = Matches::default();
                let help_text = e.to_string();
                matches.args.insert(
                    "help".to_string(),
                    ArgData {
                        value: Value::String(help_text),
                        occurrences: 0,
                    },
                );
                Ok(matches)
            }
            ErrorKind::DisplayVersion => {
                let mut matches = Matches::default();
                matches
                    .args
                    .insert("version".to_string(), Default::default());
                Ok(matches)
            }
            _ => Err(e.into()),
        },
    }
}

fn get_matches_internal(config: &Config, matches: &ArgMatches) -> Matches {
    let mut cli_matches = Matches::default();
    map_matches(config, matches, &mut cli_matches);

    if let Some((subcommand_name, subcommand_matches)) = matches.subcommand() {
        if let Some(subcommand_config) = config
            .subcommands
            .as_ref()
            .and_then(|s| s.get(subcommand_name))
        {
            cli_matches.set_subcommand(
                subcommand_name.to_string(),
                get_matches_internal(subcommand_config, subcommand_matches),
            );
        }
    }

    cli_matches
}

fn map_matches(config: &Config, matches: &ArgMatches, cli_matches: &mut Matches) {
    if let Some(args) = config.args() {
        for arg in args {
            let (occurrences, value) = if arg.takes_value {
                if arg.multiple {
                    matches
                        .get_many::<String>(&arg.name)
                        .map(|v| {
                            let mut values = Vec::new();
                            for value in v {
                                values.push(Value::String(value.into()));
                            }
                            (values.len() as u8, Value::Array(values))
                        })
                        .unwrap_or((0, Value::Null))
                } else {
                    matches
                        .get_one::<String>(&arg.name)
                        .map(|v| (1, Value::String(v.clone())))
                        .unwrap_or((0, Value::Null))
                }
            } else {
                let occurrences = matches.get_count(&arg.name);
                (occurrences, Value::Bool(occurrences > 0))
            };

            cli_matches.set_arg(arg.name.clone(), ArgData { value, occurrences });
        }
    }
}

fn get_app(
    package_info: &PackageInfo,
    version: String,
    command_name: String,
    about: Option<&String>,
    config: &Config,
) -> Command {
    let mut app = Command::new(command_name)
        .author(package_info.authors)
        .version(version.clone());

    if let Some(about) = about {
        app = app.about(about);
    }
    if let Some(long_description) = config.long_description() {
        app = app.long_about(long_description);
    }
    if let Some(before_help) = config.before_help() {
        app = app.before_help(before_help);
    }
    if let Some(after_help) = config.after_help() {
        app = app.after_help(after_help);
    }

    if let Some(args) = config.args() {
        for arg in args {
            app = app.arg(get_arg(arg.name.clone(), arg));
        }
    }

    if let Some(subcommands) = config.subcommands() {
        for (subcommand_name, subcommand) in subcommands {
            let clap_subcommand = get_app(
                package_info,
                version.clone(),
                subcommand_name.to_string(),
                subcommand.description(),
                subcommand,
            );
            app = app.subcommand(clap_subcommand);
        }
    }

    app
}

fn get_arg(arg_name: String, arg: &Arg) -> ClapArg {
    let mut clap_arg = ClapArg::new(arg_name.clone());

    if arg.index.is_none() {
        clap_arg = clap_arg.long(arg_name);
        if let Some(short) = arg.short {
            clap_arg = clap_arg.short(short);
        }
    }

    clap_arg = bind_string_arg!(arg, clap_arg, description, help);
    clap_arg = bind_string_arg!(arg, clap_arg, long_description, long_help);

    let action = if arg.multiple {
        ArgAction::Append
    } else if arg.takes_value {
        ArgAction::Set
    } else {
        ArgAction::Count
    };

    clap_arg = clap_arg.action(action);

    clap_arg = bind_value_arg!(arg, clap_arg, number_of_values);

    if let Some(values) = &arg.possible_values {
        clap_arg = clap_arg.value_parser(PossibleValuesParser::new(
            values
                .iter()
                .map(PossibleValue::new)
                .collect::<Vec<PossibleValue>>(),
        ));
    }

    clap_arg = match (arg.min_values, arg.max_values) {
        (Some(min), Some(max)) => clap_arg.num_args(min..=max),
        (Some(min), None) => clap_arg.num_args(min..),
        (None, Some(max)) => clap_arg.num_args(0..max),
        (None, None) => clap_arg,
    };
    clap_arg = clap_arg.required(arg.required);
    clap_arg = bind_string_arg!(
        arg,
        clap_arg,
        required_unless_present,
        required_unless_present
    );
    clap_arg = bind_string_slice_arg!(arg, clap_arg, required_unless_present_all);
    clap_arg = bind_string_slice_arg!(arg, clap_arg, required_unless_present_any);
    clap_arg = bind_string_arg!(arg, clap_arg, conflicts_with, conflicts_with);
    if let Some(value) = &arg.conflicts_with_all {
        clap_arg = clap_arg.conflicts_with_all(value);
    }
    clap_arg = bind_string_arg!(arg, clap_arg, requires, requires);
    if let Some(value) = &arg.requires_all {
        clap_arg = clap_arg.requires_all(value);
    }
    clap_arg = bind_if_arg!(arg, clap_arg, requires_if);
    clap_arg = bind_if_arg!(arg, clap_arg, required_if_eq);
    clap_arg = bind_value_arg!(arg, clap_arg, require_equals);
    clap_arg = bind_value_arg!(arg, clap_arg, index);

    clap_arg
}
