// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use arboard::ImageData;
use image::ImageEncoder;
use serde::de::DeserializeOwned;
use tauri::{image::Image, plugin::PluginApi, AppHandle, Manager, ResourceTable, Runtime};

use crate::models::*;

use std::{borrow::Cow, sync::Mutex};

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Clipboard<R>> {
    Ok(Clipboard {
        app: app.clone(),
        clipboard: arboard::Clipboard::new().map(Mutex::new),
    })
}

/// Access to the clipboard APIs.
pub struct Clipboard<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    clipboard: Result<Mutex<arboard::Clipboard>, arboard::Error>,
}

impl<R: Runtime> Clipboard<R> {
    pub fn write_text(&self, kind: ClipKind) -> crate::Result<()> {
        match kind {
            ClipKind::PlainText { text, .. } => match &self.clipboard {
                Ok(clipboard) => clipboard.lock().unwrap().set_text(text).map_err(Into::into),
                Err(e) => Err(crate::Error::Clipboard(e.to_string())),
            },
            _ => Err(crate::Error::Clipboard("Invalid clip kind".to_string())),
        }
    }

    pub(crate) fn write_image_inner(
        &self,
        kind: ClipKind,
        resources_table: &ResourceTable,
    ) -> crate::Result<()> {
        match kind {
            ClipKind::Image { image, .. } => match &self.clipboard {
                Ok(clipboard) => {
                    let image = image.into_img(resources_table)?;
                    clipboard
                        .lock()
                        .unwrap()
                        .set_image(ImageData {
                            bytes: Cow::Borrowed(image.rgba()),
                            width: image.width() as usize,
                            height: image.height() as usize,
                        })
                        .map_err(Into::into)
                }
                Err(e) => Err(crate::Error::Clipboard(e.to_string())),
            },
            _ => Err(crate::Error::Clipboard("Invalid clip kind".to_string())),
        }
    }

    pub fn write_image(&self, kind: ClipKind) -> crate::Result<()> {
        let resources_table = self.app.resources_table();
        self.write_image_inner(kind, &resources_table)
    }

    pub fn read_text(&self) -> crate::Result<ClipboardContents> {
        match &self.clipboard {
            Ok(clipboard) => {
                let text = clipboard.lock().unwrap().get_text()?;
                Ok(ClipboardContents::PlainText { text })
            }
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    pub fn write_html(&self, kind: ClipKind) -> crate::Result<()> {
        match kind {
            ClipKind::Html { html, alt_html, .. } => match &self.clipboard {
                Ok(clipboard) => clipboard
                    .lock()
                    .unwrap()
                    .set_html(html, alt_html)
                    .map_err(Into::into),
                Err(e) => Err(crate::Error::Clipboard(e.to_string())),
            },
            _ => Err(crate::Error::Clipboard("Invalid clip kind!".to_string())),
        }
    }

    pub fn clear(&self) -> crate::Result<()> {
        match &self.clipboard {
            Ok(clipboard) => clipboard.lock().unwrap().clear().map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    pub fn read_image(&self) -> crate::Result<Image<'_>> {
        match &self.clipboard {
            Ok(clipboard) => {
                let image = clipboard.lock().unwrap().get_image()?;

                let mut buffer: Vec<u8> = Vec::new();
                image::codecs::png::PngEncoder::new(&mut buffer).write_image(
                    &image.bytes,
                    image.width as u32,
                    image.height as u32,
                    image::ColorType::Rgba8,
                )?;

                let image = Image::new_owned(buffer, image.width as u32, image.height as u32);
                Ok(image)
            }
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }
}
