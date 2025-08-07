// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{AppHandle, Runtime, command};

use crate::{Result, SecureStorageExt};

#[command]
pub(crate) fn set_string<R: Runtime>(app: AppHandle<R>, key: &str, value: &str) -> Result<()> {
    app.secure_storage().set_string(key, value)
}

#[command]
pub(crate) fn get_string<R: Runtime>(app: AppHandle<R>, key: &str) -> Result<String> {
    app.secure_storage().get_string(key)
}

#[command]
pub(crate) fn set_binary<R: Runtime>(app: AppHandle<R>, key: &str, value: &[u8]) -> Result<()> {
    app.secure_storage().set_binary(key, value)
}

#[command]
pub(crate) fn get_binary<R: Runtime>(app: AppHandle<R>, key: &str) -> Result<Vec<u8>> {
    app.secure_storage().get_binary(key)
}
