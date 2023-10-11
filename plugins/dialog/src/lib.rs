// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/dialog/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/dialog)
//!
//! Native system dialogs for opening and saving files along with message dialogs.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

use std::{
    path::{Path, PathBuf},
    sync::mpsc::sync_channel,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::*;
#[cfg(mobile)]
use mobile::*;

macro_rules! blocking_fn {
    ($self:ident, $fn:ident) => {{
        let (tx, rx) = sync_channel(0);
        let cb = move |response| {
            tx.send(response).unwrap();
        };
        $self.$fn(cb);
        rx.recv().unwrap()
    }};
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the dialog APIs.
pub trait DialogExt<R: Runtime> {
    fn dialog(&self) -> &Dialog<R>;
}

impl<R: Runtime, T: Manager<R>> crate::DialogExt<R> for T {
    fn dialog(&self) -> &Dialog<R> {
        self.state::<Dialog<R>>().inner()
    }
}

impl<R: Runtime> Dialog<R> {
    pub fn message(&self, message: impl Into<String>) -> MessageDialogBuilder<R> {
        MessageDialogBuilder::new(
            self.clone(),
            self.app_handle().package_info().name.clone(),
            message,
        )
    }

    pub fn file(&self) -> FileDialogBuilder<R> {
        FileDialogBuilder::new(self.clone())
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    #[allow(unused_mut)]
    let mut builder = Builder::new("dialog");

    // Dialogs are implemented natively on Android
    #[cfg(not(target_os = "android"))]
    {
        let mut init_script = include_str!("init.js").to_string();
        init_script.push_str(include_str!("api-iife.js"));
        builder = builder.js_init_script(init_script);
    }
    #[cfg(target_os = "android")]
    {
        builder = builder.js_init_script(include_str!("api-iife.js").to_string());
    }

    builder
        .invoke_handler(tauri::generate_handler![
            commands::open,
            commands::save,
            commands::message,
            commands::ask,
            commands::confirm
        ])
        .setup(|app, api| {
            #[cfg(mobile)]
            let dialog = mobile::init(app, api)?;
            #[cfg(desktop)]
            let dialog = desktop::init(app, api)?;
            app.manage(dialog);
            Ok(())
        })
        .build()
}

/// A builder for message dialogs.
pub struct MessageDialogBuilder<R: Runtime> {
    #[allow(dead_code)]
    pub(crate) dialog: Dialog<R>,
    pub(crate) title: String,
    pub(crate) message: String,
    pub(crate) kind: MessageDialogKind,
    pub(crate) ok_button_label: Option<String>,
    pub(crate) cancel_button_label: Option<String>,
    #[cfg(desktop)]
    pub(crate) parent: Option<raw_window_handle::RawWindowHandle>,
}

/// Payload for the message dialog mobile API.
#[cfg(mobile)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MessageDialogPayload<'a> {
    title: &'a String,
    message: &'a String,
    kind: &'a MessageDialogKind,
    ok_button_label: &'a Option<String>,
    cancel_button_label: &'a Option<String>,
}

// raw window handle :(
unsafe impl<R: Runtime> Send for MessageDialogBuilder<R> {}

impl<R: Runtime> MessageDialogBuilder<R> {
    /// Creates a new message dialog builder.
    pub fn new(dialog: Dialog<R>, title: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            dialog,
            title: title.into(),
            message: message.into(),
            kind: Default::default(),
            ok_button_label: None,
            cancel_button_label: None,
            #[cfg(desktop)]
            parent: None,
        }
    }

    #[cfg(mobile)]
    pub(crate) fn payload(&self) -> MessageDialogPayload<'_> {
        MessageDialogPayload {
            title: &self.title,
            message: &self.message,
            kind: &self.kind,
            ok_button_label: &self.ok_button_label,
            cancel_button_label: &self.cancel_button_label,
        }
    }

    /// Sets the dialog title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set parent windows explicitly (optional)
    ///
    /// ## Platform-specific
    ///
    /// - **Linux:** Unsupported.
    #[cfg(desktop)]
    pub fn parent<W: raw_window_handle::HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.parent.replace(parent.raw_window_handle());
        self
    }

    /// Sets the label for the OK button.
    pub fn ok_button_label(mut self, label: impl Into<String>) -> Self {
        self.ok_button_label.replace(label.into());
        self
    }

    /// Sets the label for the Cancel button.
    pub fn cancel_button_label(mut self, label: impl Into<String>) -> Self {
        self.cancel_button_label.replace(label.into());
        self
    }

    /// Set type of a dialog.
    ///
    /// Depending on the system it can result in type specific icon to show up,
    /// the will inform user it message is a error, warning or just information.
    pub fn kind(mut self, kind: MessageDialogKind) -> Self {
        self.kind = kind;
        self
    }

    /// Shows a message dialog
    pub fn show<F: FnOnce(bool) + Send + 'static>(self, f: F) {
        show_message_dialog(self, f)
    }

    /// Shows a message dialog.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    pub fn blocking_show(self) -> bool {
        blocking_fn!(self, show)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileResponse {
    pub base64_data: Option<String>,
    pub duration: Option<u64>,
    pub height: Option<usize>,
    pub width: Option<usize>,
    pub mime_type: Option<String>,
    pub modified_at: Option<u64>,
    pub name: Option<String>,
    pub path: PathBuf,
    pub size: u64,
}

impl FileResponse {
    #[cfg(desktop)]
    fn new(path: PathBuf) -> Self {
        Self {
            base64_data: None,
            duration: None,
            height: None,
            width: None,
            mime_type: None,
            modified_at: None,
            name: path.file_name().map(|f| f.to_string_lossy().into_owned()),
            path,
            size: 0,
        }
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct Filter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// The file dialog builder.
///
/// Constructs file picker dialogs that can select single/multiple files or directories.
#[derive(Debug)]
pub struct FileDialogBuilder<R: Runtime> {
    #[allow(dead_code)]
    pub(crate) dialog: Dialog<R>,
    pub(crate) filters: Vec<Filter>,
    pub(crate) starting_directory: Option<PathBuf>,
    pub(crate) file_name: Option<String>,
    pub(crate) title: Option<String>,
    #[cfg(desktop)]
    pub(crate) parent: Option<raw_window_handle::RawWindowHandle>,
}

#[cfg(mobile)]
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileDialogPayload<'a> {
    filters: &'a Vec<Filter>,
    multiple: bool,
}

// raw window handle :(
unsafe impl<R: Runtime> Send for FileDialogBuilder<R> {}

impl<R: Runtime> FileDialogBuilder<R> {
    /// Gets the default file dialog builder.
    pub fn new(dialog: Dialog<R>) -> Self {
        Self {
            dialog,
            filters: Vec::new(),
            starting_directory: None,
            file_name: None,
            title: None,
            #[cfg(desktop)]
            parent: None,
        }
    }

    #[cfg(mobile)]
    pub(crate) fn payload(&self, multiple: bool) -> FileDialogPayload<'_> {
        FileDialogPayload {
            filters: &self.filters,
            multiple,
        }
    }

    /// Add file extension filter. Takes in the name of the filter, and list of extensions
    #[must_use]
    pub fn add_filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filters.push(Filter {
            name: name.into(),
            extensions: extensions.iter().map(|e| e.to_string()).collect(),
        });
        self
    }

    /// Set starting directory of the dialog.
    #[must_use]
    pub fn set_directory<P: AsRef<Path>>(mut self, directory: P) -> Self {
        self.starting_directory.replace(directory.as_ref().into());
        self
    }

    /// Set starting file name of the dialog.
    #[must_use]
    pub fn set_file_name(mut self, file_name: impl Into<String>) -> Self {
        self.file_name.replace(file_name.into());
        self
    }

    /// Sets the parent window of the dialog.
    #[cfg(desktop)]
    #[must_use]
    pub fn set_parent<W: raw_window_handle::HasRawWindowHandle>(mut self, parent: &W) -> Self {
        self.parent.replace(parent.raw_window_handle());
        self
    }

    /// Set the title of the dialog.
    #[must_use]
    pub fn set_title(mut self, title: impl Into<String>) -> Self {
        self.title.replace(title.into());
        self
    }

    /// Shows the dialog to select a single file.
    /// This is not a blocking operation,
    /// and should be used when running on the main thread to avoid deadlocks with the event loop.
    ///
    /// For usage in other contexts such as commands, prefer [`Self::pick_file`].
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// tauri::Builder::default()
    ///   .build(tauri::generate_context!("test/tauri.conf.json"))
    ///   .expect("failed to build tauri app")
    ///   .run(|app, _event| {
    ///     app.dialog().file().pick_file(|file_path| {
    ///       // do something with the optional file path here
    ///       // the file path is `None` if the user closed the dialog
    ///     })
    ///   })
    /// ```
    pub fn pick_file<F: FnOnce(Option<FileResponse>) + Send + 'static>(self, f: F) {
        #[cfg(desktop)]
        let f = |path: Option<PathBuf>| f(path.map(FileResponse::new));
        pick_file(self, f)
    }

    /// Shows the dialog to select multiple files.
    /// This is not a blocking operation,
    /// and should be used when running on the main thread to avoid deadlocks with the event loop.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// tauri::Builder::default()
    ///   .build(tauri::generate_context!("test/tauri.conf.json"))
    ///   .expect("failed to build tauri app")
    ///   .run(|app, _event| {
    ///     app.dialog().file().pick_files(|file_paths| {
    ///       // do something with the optional file paths here
    ///       // the file paths value is `None` if the user closed the dialog
    ///     })
    ///   })
    /// ```
    pub fn pick_files<F: FnOnce(Option<Vec<FileResponse>>) + Send + 'static>(self, f: F) {
        #[cfg(desktop)]
        let f = |paths: Option<Vec<PathBuf>>| {
            f(paths.map(|p| {
                p.into_iter()
                    .map(FileResponse::new)
                    .collect::<Vec<FileResponse>>()
            }))
        };
        pick_files(self, f)
    }

    /// Shows the dialog to select a single folder.
    /// This is not a blocking operation,
    /// and should be used when running on the main thread to avoid deadlocks with the event loop.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// tauri::Builder::default()
    ///   .build(tauri::generate_context!("test/tauri.conf.json"))
    ///   .expect("failed to build tauri app")
    ///   .run(|app, _event| {
    ///     app.dialog().file().pick_folder(|folder_path| {
    ///       // do something with the optional folder path here
    ///       // the folder path is `None` if the user closed the dialog
    ///     })
    ///   })
    /// ```
    #[cfg(desktop)]
    pub fn pick_folder<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
        pick_folder(self, f)
    }

    /// Shows the dialog to select multiple folders.
    /// This is not a blocking operation,
    /// and should be used when running on the main thread to avoid deadlocks with the event loop.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// tauri::Builder::default()
    ///   .build(tauri::generate_context!("test/tauri.conf.json"))
    ///   .expect("failed to build tauri app")
    ///   .run(|app, _event| {
    ///     app.dialog().file().pick_folders(|file_paths| {
    ///       // do something with the optional folder paths here
    ///       // the folder paths value is `None` if the user closed the dialog
    ///     })
    ///   })
    /// ```
    #[cfg(desktop)]
    pub fn pick_folders<F: FnOnce(Option<Vec<PathBuf>>) + Send + 'static>(self, f: F) {
        pick_folders(self, f)
    }

    /// Shows the dialog to save a file.
    ///
    /// This is not a blocking operation,
    /// and should be used when running on the main thread to avoid deadlocks with the event loop.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// tauri::Builder::default()
    ///   .build(tauri::generate_context!("test/tauri.conf.json"))
    ///   .expect("failed to build tauri app")
    ///   .run(|app, _event| {
    ///     app.dialog().file().save_file(|file_path| {
    ///       // do something with the optional file path here
    ///       // the file path is `None` if the user closed the dialog
    ///     })
    ///   })
    /// ```
    #[cfg(desktop)]
    pub fn save_file<F: FnOnce(Option<PathBuf>) + Send + 'static>(self, f: F) {
        save_file(self, f)
    }
}

/// Blocking APIs.
impl<R: Runtime> FileDialogBuilder<R> {
    /// Shows the dialog to select a single file.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// #[tauri::command]
    /// async fn my_command(app: tauri::AppHandle) {
    ///   let file_path = app.dialog().file().blocking_pick_file();
    ///   // do something with the optional file path here
    ///   // the file path is `None` if the user closed the dialog
    /// }
    /// ```
    pub fn blocking_pick_file(self) -> Option<FileResponse> {
        blocking_fn!(self, pick_file)
    }

    /// Shows the dialog to select multiple files.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// #[tauri::command]
    /// async fn my_command(app: tauri::AppHandle) {
    ///   let file_path = app.dialog().file().blocking_pick_files();
    ///   // do something with the optional file paths here
    ///   // the file paths value is `None` if the user closed the dialog
    /// }
    /// ```
    pub fn blocking_pick_files(self) -> Option<Vec<FileResponse>> {
        blocking_fn!(self, pick_files)
    }

    /// Shows the dialog to select a single folder.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// #[tauri::command]
    /// async fn my_command(app: tauri::AppHandle) {
    ///   let folder_path = app.dialog().file().blocking_pick_folder();
    ///   // do something with the optional folder path here
    ///   // the folder path is `None` if the user closed the dialog
    /// }
    /// ```
    #[cfg(desktop)]
    pub fn blocking_pick_folder(self) -> Option<PathBuf> {
        blocking_fn!(self, pick_folder)
    }

    /// Shows the dialog to select multiple folders.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// #[tauri::command]
    /// async fn my_command(app: tauri::AppHandle) {
    ///   let folder_paths = app.dialog().file().blocking_pick_folders();
    ///   // do something with the optional folder paths here
    ///   // the folder paths value is `None` if the user closed the dialog
    /// }
    /// ```
    #[cfg(desktop)]
    pub fn blocking_pick_folders(self) -> Option<Vec<PathBuf>> {
        blocking_fn!(self, pick_folders)
    }

    /// Shows the dialog to save a file.
    /// This is a blocking operation,
    /// and should *NOT* be used when running on the main thread context.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use tauri_plugin_dialog::DialogExt;
    /// #[tauri::command]
    /// async fn my_command(app: tauri::AppHandle) {
    ///   let file_path = app.dialog().file().blocking_save_file();
    ///   // do something with the optional file path here
    ///   // the file path is `None` if the user closed the dialog
    /// }
    /// ```
    #[cfg(desktop)]
    pub fn blocking_save_file(self) -> Option<PathBuf> {
        blocking_fn!(self, save_file)
    }
}
