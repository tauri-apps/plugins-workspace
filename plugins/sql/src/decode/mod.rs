// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "mysql")]
mod mysql;
#[cfg(feature = "postgres")]
mod postgres;
#[cfg(feature = "sqlite")]
mod sqlite;

#[cfg(feature = "mysql")]
pub(crate) use mysql::to_json;

#[cfg(feature = "postgres")]
pub(crate) use postgres::to_json;

#[cfg(feature = "sqlite")]
pub(crate) use sqlite::to_json;
