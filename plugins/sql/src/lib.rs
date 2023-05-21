// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(any(
    all(feature = "sqlite", feature = "mysql"),
    all(feature = "sqlite", feature = "postgres"),
    all(feature = "mysql", feature = "postgres")
))]
compile_error!(
    "Only one database driver can be enabled. Set the feature flag for the driver of your choice."
);

#[cfg(not(any(feature = "sqlite", feature = "mysql", feature = "postgres")))]
compile_error!(
    "Database driver not defined. Please set the feature flag for the driver of your choice."
);

mod decode;
mod plugin;
pub use plugin::*;
