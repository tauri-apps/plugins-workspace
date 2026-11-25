// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use tauri::plugin::PermissionState;

/// Where [`get_photo`](crate::Camera::get_photo) gets the image from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    /// Ask the user whether to take a picture or to pick one from the gallery.
    #[default]
    Prompt,
    /// Take a picture with the camera app.
    Camera,
    /// Pick an image from the gallery.
    Photos,
}

/// How the image is returned in [`Image::data`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ResultType {
    /// The base64 encoded JPEG.
    #[default]
    Base64,
    /// A `data:image/jpeg;base64,` URL.
    DataUrl,
    /// The `file://` URI of a JPEG file written to the app's cache (Android) or temporary (iOS)
    /// directory.
    Uri,
}

/// The camera that is selected when the camera app opens.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum CameraDirection {
    /// The camera on the back of the device.
    #[default]
    Rear,
    /// The camera facing the user.
    Front,
}

/// How the camera and gallery are presented on iOS.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PresentationStyle {
    /// Cover the whole screen.
    #[default]
    Fullscreen,
    /// Show a popover, which only differs from [`Self::Fullscreen`] on iPad.
    Popover,
}

/// Options for [`get_photo`](crate::Camera::get_photo).
///
/// Fields left as `None` use the default documented on them.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageOptions {
    /// The JPEG quality, from 0 to 100. Defaults to 90.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
    /// Let the user edit (iOS: crop) the image before it is returned. On Android this opens the
    /// image in an external editor app and fails if none is installed. Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_editing: Option<bool>,
    /// How the image is returned. Defaults to [`ResultType::Base64`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result_type: Option<ResultType>,
    /// Also save pictures taken with the camera to the device's gallery. Defaults to `false`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub save_to_gallery: Option<bool>,
    /// The maximum width of the image in pixels. The aspect ratio is preserved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// The maximum height of the image in pixels. The aspect ratio is preserved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Rotate the image so it is upright instead of relying on its EXIF orientation.
    /// Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_orientation: Option<bool>,
    /// Where to get the image from. Defaults to [`Source::Prompt`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<Source>,
    /// The camera to open. iOS only. Defaults to [`CameraDirection::Rear`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<CameraDirection>,
    /// How the camera and gallery are presented. iOS only.
    /// Defaults to [`PresentationStyle::Fullscreen`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_style: Option<PresentationStyle>,
    /// The title of the [`Source::Prompt`] dialog. Defaults to "Photo".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_label_header: Option<String>,
    /// The label of the cancel button of the [`Source::Prompt`] dialog. iOS only.
    /// Defaults to "Cancel".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_label_cancel: Option<String>,
    /// The label of the [`Source::Prompt`] option that opens the gallery.
    /// Defaults to "From Photos".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_label_photo: Option<String>,
    /// The label of the [`Source::Prompt`] option that opens the camera.
    /// Defaults to "Take Picture".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_label_picture: Option<String>,
}

/// Options for [`pick_images`](crate::Camera::pick_images).
///
/// Fields left as `None` use the default documented on them.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GalleryOptions {
    /// The JPEG quality, from 0 to 100. Defaults to 90.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<u8>,
    /// The maximum width of the images in pixels. The aspect ratio is preserved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    /// The maximum height of the images in pixels. The aspect ratio is preserved.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    /// Rotate the images so they are upright instead of relying on their EXIF orientation.
    /// Defaults to `true`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub correct_orientation: Option<bool>,
    /// The maximum number of images the user can select. Defaults to 0, which means no limit.
    /// Ignored on Android 12 and older.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// How the gallery is presented. iOS only. Defaults to [`PresentationStyle::Fullscreen`].
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_style: Option<PresentationStyle>,
}

/// An image returned by the camera or picked from the gallery.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Image {
    /// The image, in the representation selected with [`ImageOptions::result_type`].
    pub data: String,
    /// The URL to load the image from the webview with the `asset` protocol.
    /// Only set for [`ResultType::Uri`].
    pub asset_url: Option<String>,
    /// The format of the image. Currently always `jpeg`.
    pub format: String,
    /// Whether the image was saved to the gallery, see [`ImageOptions::save_to_gallery`].
    #[serde(default)]
    pub saved: bool,
    /// The EXIF metadata of the image. The keys are platform specific.
    #[serde(default)]
    pub exif: serde_json::Value,
}

/// The permissions of the camera plugin.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PermissionType {
    /// Access to the camera.
    Camera,
    /// Saving pictures to the gallery. Picking images from the gallery needs no permission.
    Photos,
}

/// The state of the permissions of the camera plugin.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PermissionStatus {
    /// Access to the camera.
    ///
    /// On Android the camera app is used to take the picture, which only needs the `CAMERA`
    /// permission when the app declares it in its manifest. It is reported as granted otherwise.
    pub camera: PermissionState,
    /// Saving pictures to the gallery, see [`ImageOptions::save_to_gallery`].
    ///
    /// Always granted on Android 10 and newer.
    pub photos: PermissionState,
}
