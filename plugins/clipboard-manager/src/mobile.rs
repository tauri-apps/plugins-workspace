// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tauri::{
    image::Image,
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use std::borrow::Cow;

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
    /// Writes plain text to the system clipboard, without a label.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PluginInvoke`] if the underlying mobile plugin call fails.
    pub fn write_text<'a, T: Into<Cow<'a, str>>>(&self, text: T) -> crate::Result<()> {
        let text = text.into().to_string();
        self.0
            .run_mobile_plugin("writeText", ClipKind::PlainText { text, label: None })
            .map_err(Into::into)
    }

    /// Writes plain text to the system clipboard along with a label describing the content.
    ///
    /// The label is only used on Android (it becomes the `ClipData` label); iOS ignores it.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PluginInvoke`] if the underlying mobile plugin call fails.
    pub fn write_text_with_label<'a, T: Into<Cow<'a, str>>>(
        &self,
        text: T,
        label: T,
    ) -> crate::Result<()> {
        let text = text.into().to_string();
        let label = label.into().to_string();
        self.0
            .run_mobile_plugin(
                "writeText",
                ClipKind::PlainText {
                    text,
                    label: Some(label),
                },
            )
            .map_err(Into::into)
    }

    /// Not supported on mobile.
    ///
    /// # Errors
    ///
    /// Always returns [`crate::Error::Clipboard`].
    pub fn write_image(&self, _image: &Image<'_>) -> crate::Result<()> {
        Err(crate::Error::Clipboard(
            "Unsupported on this platform".to_string(),
        ))
    }

    /// Reads the system clipboard as plain text.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PluginInvoke`] if the underlying mobile plugin call fails.
    pub fn read_text(&self) -> crate::Result<String> {
        self.0
            .run_mobile_plugin("readText", ())
            .map(|c| match c {
                ClipboardContents::PlainText { text } => text,
            })
            .map_err(Into::into)
    }

    /// Not supported on mobile.
    ///
    /// # Errors
    ///
    /// Always returns [`crate::Error::Clipboard`].
    pub fn read_image(&self) -> crate::Result<Image<'_>> {
        Err(crate::Error::Clipboard(
            "Unsupported on this platform".to_string(),
        ))
    }

    pub fn read_image_png(&self) -> crate::Result<Vec<u8>> {
        Err(crate::Error::Clipboard(
            "Unsupported on this platform".to_string(),
        ))
    }

    // Treat HTML as unsupported on mobile until tested
    /// Not supported on mobile.
    ///
    /// # Errors
    ///
    /// Always returns [`crate::Error::Clipboard`].
    pub fn write_html<'a, T: Into<Cow<'a, str>>>(
        &self,
        _html: T,
        _alt_text: Option<T>,
    ) -> crate::Result<()> {
        Err(crate::Error::Clipboard(
            "Unsupported on this platform".to_string(),
        ))
    }

    /// Clears the system clipboard.
    ///
    /// On Android this only works on SDK 28 and above; on older versions the clipboard is
    /// instead overwritten with an empty string.
    ///
    /// # Errors
    ///
    /// Returns [`crate::Error::PluginInvoke`] if the underlying mobile plugin call fails.
    pub fn clear(&self) -> crate::Result<()> {
        self.0.run_mobile_plugin("clear", ()).map_err(Into::into)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
enum ClipKind {
    PlainText { label: Option<String>, text: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum ClipboardContents {
    PlainText { text: String },
}
