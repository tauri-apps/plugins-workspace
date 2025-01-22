// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use arboard::ImageData;
use serde::de::DeserializeOwned;
use tauri::{image::Image, plugin::PluginApi, AppHandle, Runtime};

use std::{borrow::Cow, sync::Mutex};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Clipboard<R>> {
    Ok(Clipboard {
        app: app.clone(),
        clipboard: arboard::Clipboard::new().map(|c| Mutex::new(Some(c))),
    })
}

/// Access to the clipboard APIs.
pub struct Clipboard<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    // According to arboard docs the clipboard must be dropped before exit.
    // Since tauri doesn't call drop on exit we'll use an Option to take() on RunEvent::Exit.
    clipboard: Result<Mutex<Option<arboard::Clipboard>>, arboard::Error>,
}

impl<R: Runtime> Clipboard<R> {
    pub fn write_text<'a, T: Into<Cow<'a, str>>>(&self, text: T) -> crate::Result<()> {
        match &self.clipboard {
            Ok(clipboard) => clipboard
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .set_text(text)
                .map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    pub fn write_image(&self, image: &Image<'_>) -> crate::Result<()> {
        match &self.clipboard {
            Ok(clipboard) => clipboard
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .set_image(ImageData {
                    bytes: Cow::Borrowed(image.rgba()),
                    width: image.width() as usize,
                    height: image.height() as usize,
                })
                .map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    /// Warning: This method should not be used on the main thread! Otherwise the underlying libraries may deadlock on Linux, freezing the whole app, when trying to copy data copied from this app, for example if the user copies text from the WebView.
    pub fn read_text(&self) -> crate::Result<String> {
        match &self.clipboard {
            Ok(clipboard) => {
                let text = clipboard.lock().unwrap().as_mut().unwrap().get_text()?;
                Ok(text)
            }
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    pub fn write_html<'a, T: Into<Cow<'a, str>>>(
        &self,
        html: T,
        alt_text: Option<T>,
    ) -> crate::Result<()> {
        match &self.clipboard {
            Ok(clipboard) => clipboard
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .set_html(html, alt_text)
                .map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    pub fn clear(&self) -> crate::Result<()> {
        match &self.clipboard {
            Ok(clipboard) => clipboard
                .lock()
                .unwrap()
                .as_mut()
                .unwrap()
                .clear()
                .map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    /// Warning: This method should not be used on the main thread! Otherwise the underlying libraries may deadlock on Linux, freezing the whole app, when trying to copy data copied from this app, for example if the user copies text from the WebView.
    pub fn read_image(&self) -> crate::Result<Image<'_>> {
        match &self.clipboard {
            Ok(clipboard) => {
                let image = clipboard.lock().unwrap().as_mut().unwrap().get_image()?;
                let image = Image::new_owned(
                    image.bytes.to_vec(),
                    image.width as u32,
                    image.height as u32,
                );
                Ok(image)
            }
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    pub(crate) fn cleanup(&self) {
        if let Ok(clipboard) = &self.clipboard {
            clipboard.lock().unwrap().take();
        }
    }
}
