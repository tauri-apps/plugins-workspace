// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "mysql")]
pub(crate) mod mysql;
#[cfg(feature = "postgres")]
pub(crate) mod postgres;
#[cfg(feature = "sqlite")]
pub(crate) mod sqlite;
