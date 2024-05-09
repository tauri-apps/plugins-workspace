// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, image::JsImage, AppHandle, Manager, ResourceId, Runtime, State, Webview};

use crate::{Clipboard, Result};

#[command]
#[cfg(desktop)]
pub(crate) async fn write_text<R: Runtime>(
    _app: AppHandle<R>,
    clipboard: State<'_, Clipboard<R>>,
    text: &str,
    #[allow(unused)] label: Option<String>,
) -> Result<()> {
    clipboard.write_text(text)
}

#[command]
#[cfg(not(desktop))]
pub(crate) async fn write_text<R: Runtime>(
    _app: AppHandle<R>,
    clipboard: State<'_, Clipboard<R>>,
    text: &str,
    #[allow(unused)] label: Option<&str>,
) -> Result<()> {
    match label {
        Some(label) => clipboard.write_text_with_label(text, label),
        None => clipboard.write_text(text),
    }
}

#[command]
pub(crate) async fn read_text<R: Runtime>(
    _app: AppHandle<R>,
    clipboard: State<'_, Clipboard<R>>,
) -> Result<String> {
    clipboard.read_text()
}

#[command]
pub(crate) async fn write_image<R: Runtime>(
    webview: Webview<R>,
    clipboard: State<'_, Clipboard<R>>,
    image: JsImage,
) -> Result<()> {
    let resources_table = webview.resources_table();
    let image = image.into_img(&resources_table)?;
    clipboard.write_image(&image)
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
    html: &str,
    alt_text: Option<&str>,
) -> Result<()> {
    clipboard.write_html(html, alt_text)
}

#[command]
pub(crate) async fn clear<R: Runtime>(
    _app: AppHandle<R>,
    clipboard: State<'_, Clipboard<R>>,
) -> Result<()> {
    clipboard.clear()
}
