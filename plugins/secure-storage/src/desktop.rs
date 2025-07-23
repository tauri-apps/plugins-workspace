// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use keyring::Entry;
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::Result;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<SecureStorage<R>> {
    Ok(SecureStorage(app.clone()))
}

/// Access to the secure-storage APIs.
pub struct SecureStorage<R: Runtime>(AppHandle<R>);

impl<R: Runtime> SecureStorage<R> {
    pub fn set_string(&self, key: &str, value: &str) -> Result<()> {
        Ok(Entry::new(&self.0.config().identifier, key)?.set_password(value)?)
    }

    pub fn get_string(&self, key: &str) -> Result<String> {
        Ok(Entry::new(&self.0.config().identifier, key)?.get_password()?)
    }

    pub fn set_binary(&self, key: &str, value: &[u8]) -> Result<()> {
        Ok(Entry::new(&self.0.config().identifier, key)?.set_secret(value)?)
    }

    pub fn get_binary(&self, key: &str) -> Result<Vec<u8>> {
        Ok(Entry::new(&self.0.config().identifier, key)?.get_secret()?)
    }
}
