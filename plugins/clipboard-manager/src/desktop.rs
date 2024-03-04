// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

use std::sync::Mutex;

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
            ClipKind::PlainText { text, .. } => match &self.clipboard {
                Ok(clipboard) => clipboard.lock().unwrap().set_text(text).map_err(Into::into),
                Err(e) => Err(crate::Error::Clipboard(e.to_string())),
            },
            _ => Err(crate::Error::Clipboard("Invalid clip kind!".to_string())),
        }
    }

    pub fn read(&self) -> crate::Result<ClipboardContents> {
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
}
