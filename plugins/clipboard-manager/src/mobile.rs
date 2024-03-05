// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.clipboard";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_clipboard);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Clipboard<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "ClipboardPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_clipboard)?;
    Ok(Clipboard(handle))
}

/// Access to the clipboard APIs.
pub struct Clipboard<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Clipboard<R> {
    pub fn write(&self, kind: ClipKind) -> crate::Result<()> {
        self.0.run_mobile_plugin("write", kind).map_err(Into::into)
    }

    pub fn read(&self) -> crate::Result<ClipboardContents> {
        self.0.run_mobile_plugin("read", ()).map_err(Into::into)
    }

    // Treat HTML as unsupported on mobile until tested
    pub fn write_html(&self, _kind: ClipKind) -> crate::Result<()> {
        Err(crate::Error::Clipboard(
            "Unsupported on this platform".to_string(),
        ))
    }

    pub fn clear(&self) -> crate::Result<()> {
        Err(crate::Error::Clipboard(
            "Unsupported on this platform".to_string(),
        ))
    }
}
