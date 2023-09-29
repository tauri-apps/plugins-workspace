// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/sql/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/sql)
//!
//! Interface with SQL databases through [sqlx](https://github.com/launchbadge/sqlx). It supports the `sqlite`, `mysql` and `postgres` drivers, enabled by a Cargo feature.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

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
