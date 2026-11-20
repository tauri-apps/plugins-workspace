// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Take a photo with the camera or pick images from the gallery.
//!
//! - Supported platforms: Android and iOS.

#![cfg(mobile)]

use serde::{Deserialize, Serialize};
use tauri::{
    Manager, Runtime,
    plugin::{Builder, PluginHandle, TauriPlugin},
};

mod error;
mod models;

pub use error::{Error, Result};
pub use models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.camera";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_camera);

#[derive(Deserialize)]
struct PickImagesResponse {
    images: Vec<Image>,
}

#[derive(Serialize)]
struct RequestPermissionsPayload {
    permissions: Option<Vec<PermissionType>>,
}

/// Access to the camera APIs.
pub struct Camera<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Camera<R> {
    /// Asks the user to take a picture with the camera app or to pick an image from the gallery,
    /// depending on [`ImageOptions::source`].
    ///
    /// Returns `None` if the user cancelled.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`] if the user denied a required permission, the device has
    /// no camera or the image could not be processed.
    pub fn get_photo(&self, options: ImageOptions) -> Result<Option<Image>> {
        self.0
            .run_mobile_plugin("getPhoto", options)
            .map_err(Into::into)
    }

    /// Asks the user to pick any number of images from the gallery.
    ///
    /// The images are returned as files, see [`ResultType::Uri`]. The list is empty if the user
    /// cancelled.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`] if an image could not be processed.
    pub fn pick_images(&self, options: GalleryOptions) -> Result<Vec<Image>> {
        self.0
            .run_mobile_plugin::<PickImagesResponse>("pickImages", options)
            .map(|r| r.images)
            .map_err(Into::into)
    }

    /// Returns the state of the plugin's permissions without prompting the user.
    pub fn check_permissions(&self) -> Result<PermissionStatus> {
        self.0
            .run_mobile_plugin("checkPermissions", ())
            .map_err(Into::into)
    }

    /// Prompts the user for the given `permissions` (or all of them when `None`) and returns the
    /// resulting state.
    ///
    /// [`Self::get_photo`] requests the permissions it needs, so calling this is only required to
    /// ask for them ahead of time.
    pub fn request_permissions(
        &self,
        permissions: Option<Vec<PermissionType>>,
    ) -> Result<PermissionStatus> {
        self.0
            .run_mobile_plugin(
                "requestPermissions",
                RequestPermissionsPayload { permissions },
            )
            .map_err(Into::into)
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the camera APIs.
pub trait CameraExt<R: Runtime> {
    /// Returns the [`Camera`] instance managed by the app.
    fn camera(&self) -> &Camera<R>;
}

impl<R: Runtime, T: Manager<R>> crate::CameraExt<R> for T {
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
