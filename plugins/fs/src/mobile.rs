// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{
  plugin::{PluginApi, PluginHandle},
  AppHandle, Runtime,
};

#[cfg(target_os = "android")]
use crate::models::{WriteTextFilePayload, WriteTextFileResponse};

#[cfg(target_os = "android")]
use crate::Error::Tauri;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.fs";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_fs);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
  _app: &AppHandle<R>,
  api: PluginApi<R, C>,
) -> crate::Result<Fs<R>> {
  #[cfg(target_os = "android")]
  let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "FsPlugin").unwrap();
  #[cfg(target_os = "ios")]
  let handle = api.register_ios_plugin(init_plugin_android-intent-send)?;
  Ok(Fs(handle))
}

/// Access to the android-intent-send APIs.
pub struct Fs<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Fs<R> {
    pub fn write_text_file(&self, payload: WriteTextFilePayload) -> crate::Result<WriteTextFileResponse> {
        #[cfg(target_os = "android")]
        {
            let result = self
                .0
                .run_mobile_plugin::<WriteTextFileResponse>("writeTextFile", payload);
            match result {
                Ok(_) => Ok(WriteTextFileResponse{error: None}),
                Err(_) => Err(Tauri(tauri::Error::InvokeKey)),
            }
        }
        #[cfg(any(desktop, target_os = "ios"))]
        {
            write_file_inner(
                webview,
                &global_scope,
                &command_scope,
                path,
                data.as_bytes(),
                options,
            )
        }
    }
}
