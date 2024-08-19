// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
use std::path::PathBuf;

use serde::{de::DeserializeOwned, Deserialize};
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::{FileDialogBuilder, FileResponse, MessageDialogBuilder};

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

#[derive(Debug, Deserialize)]
struct FilePickerResponse {
    files: Vec<FileResponse>,
}

#[derive(Debug, Deserialize)]
struct SaveFileResponse {
    file: PathBuf,
}

pub fn pick_file<R: Runtime, F: FnOnce(Option<FileResponse>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || {
        let res = dialog
            .dialog
            .0
            .run_mobile_plugin::<FilePickerResponse>("showFilePicker", dialog.payload(false));
        if let Ok(response) = res {
            f(Some(response.files.into_iter().next().unwrap()))
        } else {
            f(None)
        }
    });
}

pub fn pick_files<R: Runtime, F: FnOnce(Option<Vec<FileResponse>>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || {
        let res = dialog
            .dialog
            .0
            .run_mobile_plugin::<FilePickerResponse>("showFilePicker", dialog.payload(true));
        if let Ok(response) = res {
            f(Some(response.files))
        } else {
            f(None)
        }
    });
}

pub fn save_file<R: Runtime, F: FnOnce(Option<PathBuf>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || {
        let res = dialog
            .dialog
            .0
            .run_mobile_plugin::<SaveFileResponse>("saveFileDialog", dialog.payload(false));
        if let Ok(response) = res {
            f(Some(response.file))
        } else {
            f(None)
        }
    });
}

#[derive(Debug, Deserialize)]
struct ShowMessageDialogResponse {
    #[allow(dead_code)]
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
