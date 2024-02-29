// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "android")]
pub const PLUGIN_IDENTIFIER: &str = "app.tauri.store";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_store);

use tauri::Runtime;

use crate::error::Result;
use crate::Store;
use std::collections::HashMap;
use serde_json::Value;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadStore {
    pub cache: HashMap<String, Value>
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveStore {
    pub store: String,
    pub cache: HashMap<String, Value>
}

#[cfg(mobile)]
impl<R: Runtime> Store<R> {
    pub fn save(&self) -> Result<()> {
        self.mobile_plugin_handle
            .as_ref()
            .ok_or_else(||crate::error::Error::MobilePluginHandleUnInitialized)?
            .run_mobile_plugin(
                SaveStore {
                    store: self.path.to_string_lossy().to_string(),
                    cache: self.cache.clone(),
                },
            )
            .map_err(Into::into)
    }

    pub fn load(&mut self) -> Result<()> {
        let result: Value = self
            .mobile_plugin_handle
            .as_ref()
            .ok_or_else(||crate::error::Error::MobilePluginHandleUnInitialized)?
            .run_mobile_plugin("load", self.path.to_string_lossy().to_string())?;

        let map = serde_json::from_value::<HashMap<String, Value>>(result).unwrap();
        self.cache.extend(map);

        Ok(())
    }
}
