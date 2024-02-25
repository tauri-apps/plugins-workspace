// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "android")]
pub const PLUGIN_IDENTIFIER: &str = "app.tauri.store";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_store);

use tauri::Runtime;

use crate::error::Result;
use crate::models::*;
use crate::Store;

#[cfg(mobile)]
impl<R: Runtime> Store<R> {
    pub fn save(&self) -> Result<()> {
        self.mobile_plugin
            .as_ref()
            .unwrap()
            .run_mobile_plugin(
                "save",
                SaveStore {
                    // TODO Figure out why
                    store: self.path.to_str().unwrap().to_string(),
                    cache: self.cache.clone(),
                },
            )
            .map_err(Into::into)
    }

    pub fn load(&mut self) -> Result<()> {
        let result: Result<LoadStore> = self
            .mobile_plugin
            .as_ref()
            .unwrap()
            .run_mobile_plugin("load", self.path.to_str().unwrap().to_string())
            .map_err(Into::into);

            // TODO is unwrap safe ?
        self.cache.extend(result.unwrap().cache);

        Ok(())
    }
}
