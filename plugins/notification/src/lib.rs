// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
#[cfg(mobile)]
use tauri::plugin::PluginHandle;
#[cfg(desktop)]
use tauri::AppHandle;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
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
use desktop::Notification;
#[cfg(mobile)]
use mobile::Notification;

#[derive(Debug, Default, Serialize, Deserialize)]
struct NotificationData {
    /// Notification id.
    #[serde(default)]
    id: usize,
    /// The notification title.
    title: Option<String>,
    /// The notification body.
    body: Option<String>,
    /// The notification icon.
    icon: Option<String>,
}

/// The notification builder.
#[derive(Debug)]
pub struct NotificationBuilder<R: Runtime> {
    #[cfg(desktop)]
    app: AppHandle<R>,
    #[cfg(mobile)]
    handle: PluginHandle<R>,
    pub(crate) data: NotificationData,
}

impl<R: Runtime> NotificationBuilder<R> {
    #[cfg(desktop)]
    fn new(app: AppHandle<R>) -> Self {
        Self {
            app,
            data: Default::default(),
        }
    }

    #[cfg(mobile)]
    fn new(handle: PluginHandle<R>) -> Self {
        Self {
            handle,
            data: Default::default(),
        }
    }

    /// Sets the notification title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.data.title.replace(title.into());
        self
    }

    /// Sets the notification body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.data.body.replace(body.into());
        self
    }

    /// Sets the notification icon.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.data.icon.replace(icon.into());
        self
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the notification APIs.
pub trait NotificationExt<R: Runtime> {
    fn notification(&self) -> &Notification<R>;
}

impl<R: Runtime, T: Manager<R>> crate::NotificationExt<R> for T {
    fn notification(&self) -> &Notification<R> {
        self.state::<Notification<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("notification")
        .invoke_handler(tauri::generate_handler![
            commands::notify,
            commands::request_permission,
            commands::is_permission_granted
        ])
        .js_init_script(include_str!("init.js").into())
        .setup(|app, api| {
            #[cfg(mobile)]
            let notification = mobile::init(app, api)?;
            #[cfg(desktop)]
            let notification = desktop::init(app, api)?;
            app.manage(notification);
            Ok(())
        })
        .build()
}
