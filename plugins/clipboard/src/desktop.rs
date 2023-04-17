// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, runtime::RuntimeHandle, AppHandle, ClipboardManager, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Clipboard<R>> {
    Ok(Clipboard(app.clone()))
}

/// Access to the clipboard APIs.
pub struct Clipboard<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Clipboard<R> {
    pub fn write(&self, kind: ClipKind) -> crate::Result<()> {
        let ClipKind::PlainText { text, .. } = kind;
        self.0
            .runtime_handle()
            .clipboard_manager()
            .write_text(text)
            .map_err(Into::into)
    }

    pub fn read(&self) -> crate::Result<ClipboardContents> {
        let text = self.0.runtime_handle().clipboard_manager().read_text()?;
        Ok(ClipboardContents::PlainText(text.unwrap_or_default()))
    }
}
