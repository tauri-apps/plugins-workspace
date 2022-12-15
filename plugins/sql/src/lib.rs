// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(any(
    all(feature = "sqlite", feature = "mysql"),
    all(feature = "sqlite", feature = "postgres"),
    all(feature = "sqlite", feature = "mssql"),
    all(feature = "mysql", feature = "postgres"),
    all(feature = "mysql", feature = "mssql"),
))]
compile_error!("Only one database driver can be enabled. Use `default-features = false` and set the feature flag for the driver of your choice.");

#[cfg(not(any(
    feature = "sqlite",
    feature = "mysql",
    feature = "postgres",
    feature = "mssql"
)))]
compile_error!(
    "Database driver not defined. Please set the feature flag for the driver of your choice."
);

#[cfg(any(
    feature = "sqlite",
    feature = "mssql",
    feature = "mysql",
    feature = "postgres"
))]
pub mod deserialize;

#[cfg(any(
    all(
        feature = "sqlite",
        not(any(feature = "mssql", feature = "mysql", feature = "postgres"))
    ),
    all(
        feature = "mysql",
        not(any(feature = "sqlite", feature = "mssql", feature = "postgres"))
    ),
    all(
        feature = "postgres",
        not(any(feature = "sqlite", feature = "mysql", feature = "mysql"))
    ),
    all(
        feature = "mssql",
        not(any(feature = "sqlite", feature = "mysql", feature = "postgres"))
    ),
))]
mod plugin;
#[cfg(any(
    all(
        feature = "sqlite",
        not(any(feature = "mssql", feature = "mysql", feature = "postgres"))
    ),
    all(
        feature = "mysql",
        not(any(feature = "sqlite", feature = "mssql", feature = "postgres"))
    ),
    all(
        feature = "postgres",
        not(any(feature = "sqlite", feature = "mysql", feature = "mysql"))
    ),
    all(
        feature = "mssql",
        not(any(feature = "sqlite", feature = "mysql", feature = "postgres"))
    ),
))]
pub use plugin::*;
