// Copyright 2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(any(
    all(feature = "sqlite", feature = "mysql"),
    all(feature = "sqlite", feature = "postgres"),
    all(feature = "mysql", feature = "postgres")
))]
compile_error!("Only one database driver can be enabled. Use `default-features = false` and set the feature flag for the driver of your choice.");

#[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "postgres")))]
compile_error!(
    "Database driver not defined. Please set the feature flag for the driver of your choice."
);

#[cfg(any(
    all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))),
    all(feature = "mysql", not(any(feature = "sqlite", feature = "postgres"))),
    all(feature = "postgres", not(any(feature = "sqlite", feature = "mysql"))),
))]
mod plugin;
#[cfg(any(
    all(feature = "sqlite", not(any(feature = "mysql", feature = "postgres"))),
    all(feature = "mysql", not(any(feature = "sqlite", feature = "postgres"))),
    all(feature = "postgres", not(any(feature = "sqlite", feature = "mysql"))),
))]
pub use plugin::*;
