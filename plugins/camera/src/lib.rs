// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(mobile)]

use tauri::{
  plugin::{Builder, PluginHandle, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;
mod error;
pub mod models;
pub use error::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.camera";

#[cfg(target_os = "ios")]
extern "C" {
  fn init_plugin_camera(webview: tauri::cocoa::base::id);
}

/// A helper class to access the mobile camera APIs.
pub struct Camera<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Camera<R> {
  pub fn get_photo(&self, options: ImageOptions) -> Result<Image> {
    self
      .0
      .run_mobile_plugin("getPhoto", options)
      .map_err(Into::into)
  }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the camera APIs.
pub trait CameraExt<R: Runtime> {
  fn camera(&self) -> &Camera<R>;
}

impl<R: Runtime, T: Manager<R>> CameraExt<R> for T {
  fn camera(&self) -> &Camera<R> {
    self.state::<Camera<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("camera")
    .setup(|app, api| {
      #[cfg(target_os = "android")]
      let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "CameraPlugin")?;
      #[cfg(target_os = "ios")]
      let handle = api.register_ios_plugin(init_plugin_camera)?;
      app.manage(Camera(handle));
      Ok(())
    })
    .build()
}
