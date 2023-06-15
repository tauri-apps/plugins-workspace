// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::HashMap;

use serde::Deserialize;

/// A CLI argument definition.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Arg {
    /// The short version of the argument, without the preceding -.
    ///
    /// NOTE: Any leading `-` characters will be stripped, and only the first non-character will be used as the short version.
    pub short: Option<char>,
    /// The unique argument name
    pub name: String,
    /// The argument description which will be shown on the help information.
    /// Typically, this is a short (one line) description of the arg.
    pub description: Option<String>,
    /// The argument long description which will be shown on the help information.
    /// Typically this a more detailed (multi-line) message that describes the argument.
    #[serde(alias = "long-description")]
    pub long_description: Option<String>,
    /// Specifies that the argument takes a value at run time.
    ///
    /// NOTE: values for arguments may be specified in any of the following methods
    /// - Using a space such as -o value or --option value
    /// - Using an equals and no space such as -o=value or --option=value
    /// - Use a short and no space such as -ovalue
    #[serde(default, alias = "takes-value")]
    pub takes_value: bool,
    /// Specifies that the argument may have an unknown number of multiple values. Without any other settings, this argument may appear only once.
    ///
    /// For example, --opt val1 val2 is allowed, but --opt val1 val2 --opt val3 is not.
    ///
    /// NOTE: Setting this requires `takes_value` to be set to true.
    #[serde(default)]
    pub multiple: bool,
    /// Specifies how many values are required to satisfy this argument. For example, if you had a
    /// `-f <file>` argument where you wanted exactly 3 'files' you would set
    /// `number_of_values = 3`, and this argument wouldn't be satisfied unless the user provided
    /// 3 and only 3 values.
    ///
    /// **NOTE:** Does *not* require `multiple_occurrences = true` to be set. Setting
    /// `multiple_occurrences = true` would allow `-f <file> <file> <file> -f <file> <file> <file>` where
    /// as *not* setting it would only allow one occurrence of this argument.
    ///
    /// **NOTE:** implicitly sets `takes_value = true` and `multiple_values = true`.
    #[serde(alias = "number-of-values")]
    pub number_of_values: Option<usize>,
    /// Specifies a list of possible values for this argument.
    /// At runtime, the CLI verifies that only one of the specified values was used, or fails with an error message.
    #[serde(alias = "possible-values")]
    pub possible_values: Option<Vec<String>>,
    /// Specifies the minimum number of values for this argument.
    /// For example, if you had a -f `<file>` argument where you wanted at least 2 'files',
    /// you would set `minValues: 2`, and this argument would be satisfied if the user provided, 2 or more values.
    #[serde(alias = "min-values")]
    pub min_values: Option<usize>,
    /// Specifies the maximum number of values are for this argument.
    /// For example, if you had a -f `<file>` argument where you wanted up to 3 'files',
    /// you would set .max_values(3), and this argument would be satisfied if the user provided, 1, 2, or 3 values.
    #[serde(alias = "max-values")]
    pub max_values: Option<usize>,
    /// Sets whether or not the argument is required by default.
    ///
    /// - Required by default means it is required, when no other conflicting rules have been evaluated
    /// - Conflicting rules take precedence over being required.
    #[serde(default)]
    pub required: bool,
    /// Sets an arg that override this arg's required setting
    /// i.e. this arg will be required unless this other argument is present.
    #[serde(alias = "required-unless-present")]
    pub required_unless_present: Option<String>,
    /// Sets args that override this arg's required setting
    /// i.e. this arg will be required unless all these other arguments are present.
    #[serde(alias = "required-unless-present-all")]
    pub required_unless_present_all: Option<Vec<String>>,
    /// Sets args that override this arg's required setting
    /// i.e. this arg will be required unless at least one of these other arguments are present.
    #[serde(alias = "required-unless-present-any")]
    pub required_unless_present_any: Option<Vec<String>>,
    /// Sets a conflicting argument by name
    /// i.e. when using this argument, the following argument can't be present and vice versa.
    #[serde(alias = "conflicts-with")]
    pub conflicts_with: Option<String>,
    /// The same as conflictsWith but allows specifying multiple two-way conflicts per argument.
    #[serde(alias = "conflicts-with-all")]
    pub conflicts_with_all: Option<Vec<String>>,
    /// Tets an argument by name that is required when this one is present
    /// i.e. when using this argument, the following argument must be present.
    pub requires: Option<String>,
    /// Sts multiple arguments by names that are required when this one is present
    /// i.e. when using this argument, the following arguments must be present.
    #[serde(alias = "requires-all")]
    pub requires_all: Option<Vec<String>>,
    /// Allows a conditional requirement with the signature [arg, value]
    /// the requirement will only become valid if `arg`'s value equals `${value}`.
    #[serde(alias = "requires-if")]
    pub requires_if: Option<(String, String)>,
    /// Allows specifying that an argument is required conditionally with the signature [arg, value]
    /// the requirement will only become valid if the `arg`'s value equals `${value}`.
    #[serde(alias = "required-if-eq")]
    pub required_if_eq: Option<(String, String)>,
    /// Requires that options use the --option=val syntax
    /// i.e. an equals between the option and associated value.
    #[serde(alias = "requires-equals")]
    pub require_equals: Option<bool>,
    /// The positional argument index, starting at 1.
    ///
    /// The index refers to position according to other positional argument.
    /// It does not define position in the argument list as a whole. When utilized with multiple=true,
    /// only the last positional argument may be defined as multiple (i.e. the one with the highest index).
    pub index: Option<usize>,
}

/// describes a CLI configuration
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    /// Command description which will be shown on the help information.
    pub description: Option<String>,
    /// Command long description which will be shown on the help information.
    #[serde(alias = "long-description")]
    pub long_description: Option<String>,
    /// Adds additional help information to be displayed in addition to auto-generated help.
    /// This information is displayed before the auto-generated help information.
    /// This is often used for header information.
    #[serde(alias = "before-help")]
    pub before_help: Option<String>,
    /// Adds additional help information to be displayed in addition to auto-generated help.
    /// This information is displayed after the auto-generated help information.
    /// This is often used to describe how to use the arguments, or caveats to be noted.
    #[serde(alias = "after-help")]
    pub after_help: Option<String>,
    /// List of arguments for the command
    pub args: Option<Vec<Arg>>,
    /// List of subcommands of this command
    pub subcommands: Option<HashMap<String, Config>>,
}

impl Config {
    /// List of arguments for the command
    pub fn args(&self) -> Option<&Vec<Arg>> {
        self.args.as_ref()
    }

    /// List of subcommands of this command
    pub fn subcommands(&self) -> Option<&HashMap<String, Config>> {
        self.subcommands.as_ref()
    }

    /// Command description which will be shown on the help information.
    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// Command long description which will be shown on the help information.
    pub fn long_description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    /// Adds additional help information to be displayed in addition to auto-generated help.
    /// This information is displayed before the auto-generated help information.
    /// This is often used for header information.
    pub fn before_help(&self) -> Option<&String> {
        self.before_help.as_ref()
    }

    /// Adds additional help information to be displayed in addition to auto-generated help.
    /// This information is displayed after the auto-generated help information.
    /// This is often used to describe how to use the arguments, or caveats to be noted.
    pub fn after_help(&self) -> Option<&String> {
        self.after_help.as_ref()
    }
}
