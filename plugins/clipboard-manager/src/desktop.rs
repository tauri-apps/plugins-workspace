// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use arboard::ImageData;
use image::{GenericImageView, ImageEncoder};
use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

use std::io::{BufWriter, Cursor};
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
    pub fn write(&self, kind: ClipKind) -> crate::Result<()> {
        match kind {
            ClipKind::PlainText { text, .. } => self.write_text(text),
            ClipKind::Image { buffer } => self.write_image(buffer),
        }
    }

    fn write_text(&self, text: String) -> crate::Result<()> {
        match &self.clipboard {
            Ok(clipboard) => clipboard.lock().unwrap().set_text(text).map_err(Into::into),
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }

    fn write_image(&self, buffer: Vec<u8>) -> crate::Result<()> {
        match &self.clipboard {
            Ok(clipboard) => {
                let image = buffer_to_image_data(&buffer)?;
                clipboard
                    .lock()
                    .unwrap()
                    .set_image(image)
                    .map_err(Into::into)
            }
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
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

    pub fn read_image(&self) -> crate::Result<ClipboardContents> {
        match &self.clipboard {
            Ok(clipboard) => {
                let image = clipboard.lock().unwrap().get_image()?;
                let buffer = image_data_to_buffer(&image)?;
                Ok(ClipboardContents::Image { buffer })
            }
            Err(e) => Err(crate::Error::Clipboard(e.to_string())),
        }
    }
}

fn buffer_to_image_data(buffer: &[u8]) -> crate::Result<ImageData> {
    let loaded = image::load_from_memory(buffer)?;

    let pixels = loaded
        .pixels()
        .flat_map(|(_, _, pixel)| pixel.0)
        .collect::<Vec<_>>();

    Ok(ImageData {
        width: loaded.width() as usize,
        height: loaded.height() as usize,
        bytes: Cow::Owned(pixels),
    })
}

// copied from https://github.com/CrossCopy/tauri-plugin-clipboard/blob/main/src/util.rs
fn image_data_to_buffer(img: &ImageData) -> crate::Result<Vec<u8>> {
    let mut buffer: Vec<u8> = Vec::new();
    image::codecs::png::PngEncoder::new(BufWriter::new(Cursor::new(&mut buffer))).write_image(
        &img.bytes,
        img.width as u32,
        img.height as u32,
        image::ColorType::Rgba8,
    )?;
    Ok(buffer)
}
