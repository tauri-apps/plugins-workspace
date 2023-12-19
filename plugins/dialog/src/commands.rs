// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use tauri::{command, Manager, Runtime, State, Window};
use tauri_plugin_fs::FsExt;

use crate::{Dialog, FileDialogBuilder, FileResponse, MessageDialogKind, Result};

#[derive(Serialize)]
#[serde(untagged)]
pub enum OpenResponse {
    #[cfg(desktop)]
    Folders(Option<Vec<PathBuf>>),
    #[cfg(desktop)]
    Folder(Option<PathBuf>),
    Files(Option<Vec<FileResponse>>),
    File(Option<FileResponse>),
}

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogFilter {
    name: String,
    extensions: Vec<String>,
}

/// The options for the open dialog API.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDialogOptions {
    /// The title of the dialog window.
    title: Option<String>,
    /// The filters of the dialog.
    #[serde(default)]
    filters: Vec<DialogFilter>,
    /// Whether the dialog allows multiple selection or not.
    #[serde(default)]
    multiple: bool,
    /// Whether the dialog is a directory selection (`true` value) or file selection (`false` value).
    #[serde(default)]
    directory: bool,
    /// The initial path of the dialog.
    default_path: Option<PathBuf>,
    /// If [`Self::directory`] is true, indicates that it will be read recursively later.
    /// Defines whether subdirectories will be allowed on the scope or not.
    #[serde(default)]
    #[cfg_attr(mobile, allow(dead_code))]
    recursive: bool,
}

/// The options for the save dialog API.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(mobile, allow(dead_code))]
pub struct SaveDialogOptions {
    /// The title of the dialog window.
    title: Option<String>,
    /// The filters of the dialog.
    #[serde(default)]
    filters: Vec<DialogFilter>,
    /// The initial path of the dialog.
    default_path: Option<PathBuf>,
}

fn set_default_path<R: Runtime>(
    mut dialog_builder: FileDialogBuilder<R>,
    default_path: PathBuf,
) -> FileDialogBuilder<R> {
    // we need to adjust the separator on Windows: https://github.com/tauri-apps/tauri/issues/8074
    let default_path: PathBuf = default_path.components().collect();
    if default_path.is_file() || !default_path.exists() {
        if let (Some(parent), Some(file_name)) = (default_path.parent(), default_path.file_name()) {
            if parent.components().count() > 0 {
                dialog_builder = dialog_builder.set_directory(parent);
            }
            dialog_builder = dialog_builder.set_file_name(file_name.to_string_lossy());
        } else {
            dialog_builder = dialog_builder.set_directory(default_path);
        }
        dialog_builder
    } else {
        dialog_builder.set_directory(default_path)
    }
}

#[command]
pub(crate) async fn open<R: Runtime>(
    window: Window<R>,
    dialog: State<'_, Dialog<R>>,
    options: OpenDialogOptions,
) -> Result<OpenResponse> {
    let mut dialog_builder = dialog.file();
    #[cfg(any(windows, target_os = "macos"))]
    {
        dialog_builder = dialog_builder.set_parent(&window);
    }
    if let Some(title) = options.title {
        dialog_builder = dialog_builder.set_title(title);
    }
    if let Some(default_path) = options.default_path {
        dialog_builder = set_default_path(dialog_builder, default_path);
    }
    for filter in options.filters {
        let extensions: Vec<&str> = filter.extensions.iter().map(|s| &**s).collect();
        dialog_builder = dialog_builder.add_filter(filter.name, &extensions);
    }

    let res = if options.directory {
        #[cfg(desktop)]
        {
            if options.multiple {
                let folders = dialog_builder.blocking_pick_folders();
                if let Some(folders) = &folders {
                    for folder in folders {
                        if let Some(s) = window.try_fs_scope() {
                            s.allow_directory(folder, options.recursive)?;
                        }
                    }
                }
                OpenResponse::Folders(folders)
            } else {
                let folder = dialog_builder.blocking_pick_folder();
                if let Some(path) = &folder {
                    if let Some(s) = window.try_fs_scope() {
                        s.allow_directory(path, options.recursive)?;
                    }
                }
                OpenResponse::Folder(folder)
            }
        }
        #[cfg(mobile)]
        return Err(crate::Error::FolderPickerNotImplemented);
    } else if options.multiple {
        let files = dialog_builder.blocking_pick_files();
        if let Some(files) = &files {
            for file in files {
                if let Some(s) = window.try_fs_scope() {
                    s.allow_file(&file.path)?;
                }
                window
                    .state::<tauri::scope::Scopes>()
                    .allow_file(&file.path)?;
            }
        }
        OpenResponse::Files(files)
    } else {
        let file = dialog_builder.blocking_pick_file();
        if let Some(file) = &file {
            if let Some(s) = window.try_fs_scope() {
                s.allow_file(&file.path)?;
            }
            window
                .state::<tauri::scope::Scopes>()
                .allow_file(&file.path)?;
        }
        OpenResponse::File(file)
    };
    Ok(res)
}

#[allow(unused_variables)]
#[command]
pub(crate) async fn save<R: Runtime>(
    window: Window<R>,
    dialog: State<'_, Dialog<R>>,
    options: SaveDialogOptions,
) -> Result<Option<PathBuf>> {
    #[cfg(mobile)]
    return Err(crate::Error::FileSaveDialogNotImplemented);
    #[cfg(desktop)]
    {
        let mut dialog_builder = dialog.file();
        #[cfg(any(windows, target_os = "macos"))]
        {
            dialog_builder = dialog_builder.set_parent(&window);
        }
        if let Some(title) = options.title {
            dialog_builder = dialog_builder.set_title(title);
        }
        if let Some(default_path) = options.default_path {
            dialog_builder = set_default_path(dialog_builder, default_path);
        }
        for filter in options.filters {
            let extensions: Vec<&str> = filter.extensions.iter().map(|s| &**s).collect();
            dialog_builder = dialog_builder.add_filter(filter.name, &extensions);
        }

        let path = dialog_builder.blocking_save_file();
        if let Some(p) = &path {
            if let Some(s) = window.try_fs_scope() {
                s.allow_file(p)?;
            }
            window.state::<tauri::scope::Scopes>().allow_file(p)?;
        }

        Ok(path)
    }
}

fn message_dialog<R: Runtime>(
    #[allow(unused_variables)] window: Window<R>,
    dialog: State<'_, Dialog<R>>,
    title: Option<String>,
    message: String,
    type_: Option<MessageDialogKind>,
    ok_button_label: Option<String>,
    cancel_button_label: Option<String>,
) -> bool {
    let mut builder = dialog.message(message);

    if let Some(title) = title {
        builder = builder.title(title);
    }

    #[cfg(any(windows, target_os = "macos"))]
    {
        builder = builder.parent(&window);
    }

    if let Some(type_) = type_ {
        builder = builder.kind(type_);
    }

    if let Some(ok) = ok_button_label {
        builder = builder.ok_button_label(ok);
    }

    if let Some(cancel) = cancel_button_label {
        builder = builder.cancel_button_label(cancel);
    }

    builder.blocking_show()
}

#[command]
pub(crate) async fn message<R: Runtime>(
    window: Window<R>,
    dialog: State<'_, Dialog<R>>,
    title: Option<String>,
    message: String,
    type_: Option<MessageDialogKind>,
    ok_button_label: Option<String>,
) -> Result<bool> {
    Ok(message_dialog(
        window,
        dialog,
        title,
        message,
        type_,
        ok_button_label,
        None,
    ))
}

#[command]
pub(crate) async fn ask<R: Runtime>(
    window: Window<R>,
    dialog: State<'_, Dialog<R>>,
    title: Option<String>,
    message: String,
    type_: Option<MessageDialogKind>,
    ok_button_label: Option<String>,
    cancel_button_label: Option<String>,
) -> Result<bool> {
    Ok(message_dialog(
        window,
        dialog,
        title,
        message,
        type_,
        Some(ok_button_label.unwrap_or_else(|| "Yes".into())),
        Some(cancel_button_label.unwrap_or_else(|| "No".into())),
    ))
}

#[command]
pub(crate) async fn confirm<R: Runtime>(
    window: Window<R>,
    dialog: State<'_, Dialog<R>>,
    title: Option<String>,
    message: String,
    type_: Option<MessageDialogKind>,
    ok_button_label: Option<String>,
    cancel_button_label: Option<String>,
) -> Result<bool> {
    Ok(message_dialog(
        window,
        dialog,
        title,
        message,
        type_,
        Some(ok_button_label.unwrap_or_else(|| "Ok".into())),
        Some(cancel_button_label.unwrap_or_else(|| "Cancel".into())),
    ))
}
