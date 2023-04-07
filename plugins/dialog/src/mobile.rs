// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize};
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::{FileDialogBuilder, MessageDialogBuilder};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.dialog";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_dialog);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Dialog<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "DialogPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_dialog)?;
    Ok(Dialog(handle))
}

/// Access to the dialog APIs.
#[derive(Debug)]
pub struct Dialog<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Clone for Dialog<R> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<R: Runtime> Dialog<R> {
    pub(crate) fn app_handle(&self) -> &AppHandle<R> {
        self.0.app()
    }
}

pub fn pick_file<R: Runtime, F: FnOnce(Option<PathBuf>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let res = dialog
        .dialog
        .0
        .run_mobile_plugin::<Option<PathBuf>>("pickFile", dialog.payload());
    f(res.unwrap_or_default())
}

pub fn pick_files<R: Runtime, F: FnOnce(Option<Vec<PathBuf>>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let res = dialog
        .dialog
        .0
        .run_mobile_plugin::<Option<Vec<PathBuf>>>("pickFiles", dialog.payload());
    f(res.unwrap_or_default())
}

pub fn pick_folder<R: Runtime, F: FnOnce(Option<PathBuf>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let res = dialog
        .dialog
        .0
        .run_mobile_plugin::<Option<PathBuf>>("pickFolder", dialog.payload());
    f(res.unwrap_or_default())
}

pub fn pick_folders<R: Runtime, F: FnOnce(Option<Vec<PathBuf>>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let res = dialog
        .dialog
        .0
        .run_mobile_plugin::<Option<Vec<PathBuf>>>("pickFolders", dialog.payload());
    f(res.unwrap_or_default())
}

pub fn save_file<R: Runtime, F: FnOnce(Option<PathBuf>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    let res = dialog
        .dialog
        .0
        .run_mobile_plugin::<Option<PathBuf>>("saveFile", dialog.payload());
    f(res.unwrap_or_default())
}

#[derive(Debug, Deserialize)]
struct ShowMessageDialogResponse {
    cancelled: bool,
    value: bool,
}

/// Shows a message dialog
pub fn show_message_dialog<R: Runtime, F: FnOnce(bool) + Send + 'static>(
    dialog: MessageDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || {
        let res = dialog
            .dialog
            .0
            .run_mobile_plugin::<ShowMessageDialogResponse>("showMessageDialog", dialog.payload());
        f(res.map(|r| r.value).unwrap_or_default())
    });
}
