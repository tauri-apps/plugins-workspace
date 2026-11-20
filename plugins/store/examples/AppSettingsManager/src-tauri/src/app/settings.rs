// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_plugin_store::Store;

#[derive(Debug, Clone)]
pub struct AppSettings {
    pub launch_at_login: bool,
    pub theme: String,
}

impl<R: tauri::Runtime> From<&Store<R>> for AppSettings {
    fn from(store: &Store<R>) -> Self {
        let launch_at_login = store
            .get("appSettings.launchAtLogin")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let theme = store
            .get("appSettings.theme")
            .and_then(|v| v.as_str().map(String::from))
            .unwrap_or_else(|| "dark".to_owned());

        AppSettings {
            launch_at_login,
            theme,
        }
    }
}
