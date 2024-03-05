// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::Runtime;

use crate::error::Result;
use crate::Store;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadStore {
    pub cache: HashMap<String, Value>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveStore {
    pub store: String,
    pub cache: HashMap<String, Value>,
}

#[cfg(mobile)]
impl<R: Runtime> Store<R> {
    pub fn save(&self) -> Result<()> {
        self.mobile_plugin_handle
            .as_ref()
            .ok_or_else(|| crate::error::Error::MobilePluginHandleUnInitialized)?
            .run_mobile_plugin(
                "save",
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
            .ok_or_else(|| crate::error::Error::MobilePluginHandleUnInitialized)?
            .run_mobile_plugin("load", self.path.to_string_lossy().to_string())?;

        let map = serde_json::from_value::<HashMap<String, Value>>(result)?;
        self.cache.extend(map);

        Ok(())
    }
}
