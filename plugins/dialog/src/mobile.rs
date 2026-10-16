// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize, de::DeserializeOwned};
use tauri::{
    AppHandle, Runtime, Webview,
    plugin::{PluginApi, PluginHandle, mobile::PluginInvokeError},
};

use crate::{
    FileDialogBuilder, FilePath, MessageDialogBuilder, MessageDialogResult,
    error::or_previous_result,
};

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

    fn run<T: DeserializeOwned>(
        &self,
        origin: Option<&Webview<R>>,
        command: &str,
        payload: impl Serialize,
    ) -> crate::Result<T> {
        match origin {
            Some(webview) => self
                .0
                .run_mobile_plugin_with_webview(webview, command, payload),
            None => self.0.run_mobile_plugin(command, payload),
        }
        .map_err(Into::into)
    }
}

fn plugin_error_code(error: &crate::Error) -> Option<&str> {
    match error {
        crate::Error::PluginInvoke(PluginInvokeError::InvokeRejected(error)) => {
            error.code.as_deref()
        }
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct FilePickerResponse {
    files: Option<Vec<FilePath>>,
}

#[derive(Debug, Deserialize)]
struct SaveFileResponse {
    file: Option<FilePath>,
}

pub(crate) fn blocking_pick_files<R: Runtime>(
    dialog: FileDialogBuilder<R>,
    multiple: bool,
) -> crate::Result<Option<Vec<FilePath>>> {
    let result = dialog.dialog.run(
        dialog.origin.as_ref(),
        "showFilePicker",
        dialog.payload(multiple),
    );
    or_previous_result(
        result.map(|r: FilePickerResponse| r.files),
        None,
        plugin_error_code,
    )
}

pub(crate) fn blocking_save_file<R: Runtime>(
    dialog: FileDialogBuilder<R>,
) -> crate::Result<Option<FilePath>> {
    let result = dialog.dialog.run(
        dialog.origin.as_ref(),
        "saveFileDialog",
        dialog.payload(false),
    );
    or_previous_result(
        result.map(|r: SaveFileResponse| r.file),
        None,
        plugin_error_code,
    )
}

pub fn pick_file<R: Runtime, F: FnOnce(Option<FilePath>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || {
        let files = blocking_pick_files(dialog, false).ok().flatten();
        f(files.and_then(|files| files.into_iter().next()))
    });
}

pub fn pick_files<R: Runtime, F: FnOnce(Option<Vec<FilePath>>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || f(blocking_pick_files(dialog, true).ok().flatten()));
}

pub fn save_file<R: Runtime, F: FnOnce(Option<FilePath>) + Send + 'static>(
    dialog: FileDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || f(blocking_save_file(dialog).ok().flatten()));
}

#[derive(Debug, Deserialize)]
struct ShowMessageDialogResponse {
    value: String,
}

pub(crate) fn blocking_show_message_dialog<R: Runtime>(
    dialog: MessageDialogBuilder<R>,
) -> crate::Result<MessageDialogResult> {
    let result = dialog.dialog.run(
        dialog.origin.as_ref(),
        "showMessageDialog",
        dialog.payload(),
    );
    or_previous_result(
        result.map(|r: ShowMessageDialogResponse| r.value.into()),
        MessageDialogResult::default(),
        plugin_error_code,
    )
}

/// Shows a message dialog
pub fn show_message_dialog<R: Runtime, F: FnOnce(MessageDialogResult) + Send + 'static>(
    dialog: MessageDialogBuilder<R>,
    f: F,
) {
    std::thread::spawn(move || f(blocking_show_message_dialog(dialog).unwrap_or_default()));
}
