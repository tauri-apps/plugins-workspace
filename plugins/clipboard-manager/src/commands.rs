// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, AppHandle, Manager, ResourceId, Runtime, State, Webview};

use crate::{ClipKind, Clipboard, ClipboardContents, Result};

#[command]
pub(crate) async fn write_text<R: Runtime>(
    _app: AppHandle<R>,
    clipboard: State<'_, Clipboard<R>>,
    data: ClipKind,
) -> Result<()> {
    clipboard.write_text(data)
}

#[command]
pub(crate) async fn write_image<R: Runtime>(
    webview: Webview<R>,
    clipboard: State<'_, Clipboard<R>>,
    data: ClipKind,
) -> Result<()> {
    let resources_table = webview.resources_table();
    clipboard.write_image_inner(data, &resources_table)
}

#[command]
pub(crate) async fn read_text<R: Runtime>(
    _app: AppHandle<R>,
    clipboard: State<'_, Clipboard<R>>,
) -> Result<ClipboardContents> {
    clipboard.read_text()
}

#[command]
pub(crate) async fn read_image<R: Runtime>(
    webview: Webview<R>,
    clipboard: State<'_, Clipboard<R>>,
) -> Result<ResourceId> {
    let image = clipboard.read_image()?.to_owned();
    let mut resources_table = webview.resources_table();
    let rid = resources_table.add(image);
    Ok(rid)
}

#[command]
pub(crate) async fn write_html<R: Runtime>(
    _app: AppHandle<R>,
    clipboard: State<'_, Clipboard<R>>,
    data: ClipKind,
) -> Result<()> {
    clipboard.write_html(data)
}

#[command]
pub(crate) async fn clear<R: Runtime>(
    _app: AppHandle<R>,
    clipboard: State<'_, Clipboard<R>>,
) -> Result<()> {
    clipboard.clear()
}
