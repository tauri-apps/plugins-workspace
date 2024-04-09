// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/notification/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/notification)
//!
//! Send message notifications (brief auto-expiring OS window element) to your user. Can also be used with the Notification Web API.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]

use serde::Serialize;
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

#[allow(dead_code, unused_imports, deprecated, clippy::derivable_impls)]
mod notify_rust;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Notification;
#[cfg(mobile)]
use mobile::Notification;

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

    /// Sets the notification identifier.
    pub fn id(mut self, id: i32) -> Self {
        self.data.id = id;
        self
    }

    /// Identifier of the {@link Channel} that deliveres this notification.
    ///
    /// If the channel does not exist, the notification won't fire.
    /// Make sure the channel exists with {@link listChannels} and {@link createChannel}.
    pub fn channel_id(mut self, id: impl Into<String>) -> Self {
        self.data.channel_id.replace(id.into());
        self
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

    /// Schedule this notification to fire on a later time or a fixed interval.
    pub fn schedule(mut self, schedule: Schedule) -> Self {
        self.data.schedule.replace(schedule);
        self
    }

    /// Multiline text.
    /// Changes the notification style to big text.
    /// Cannot be used with `inboxLines`.
    pub fn large_body(mut self, large_body: impl Into<String>) -> Self {
        self.data.large_body.replace(large_body.into());
        self
    }

    /// Detail text for the notification with `largeBody`, `inboxLines` or `groupSummary`.
    pub fn summary(mut self, summary: impl Into<String>) -> Self {
        self.data.summary.replace(summary.into());
        self
    }

    /// Defines an action type for this notification.
    pub fn action_type_id(mut self, action_type_id: impl Into<String>) -> Self {
        self.data.action_type_id.replace(action_type_id.into());
        self
    }

    /// Identifier used to group multiple notifications.
    ///
    /// https://developer.apple.com/documentation/usernotifications/unmutablenotificationcontent/1649872-threadidentifier
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.data.group.replace(group.into());
        self
    }

    /// Instructs the system that this notification is the summary of a group on Android.
    pub fn group_summary(mut self) -> Self {
        self.data.group_summary = true;
        self
    }

    /// The sound resource name. Only available on mobile.
    pub fn sound(mut self, sound: impl Into<String>) -> Self {
        self.data.sound.replace(sound.into());
        self
    }

    /// Append an inbox line to the notification.
    /// Changes the notification style to inbox.
    /// Cannot be used with `largeBody`.
    ///
    /// Only supports up to 5 lines.
    pub fn inbox_line(mut self, line: impl Into<String>) -> Self {
        self.data.inbox_lines.push(line.into());
        self
    }

    /// Notification icon.
    ///
    /// On Android the icon must be placed in the app's `res/drawable` folder.
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.data.icon.replace(icon.into());
        self
    }

    /// Notification large icon (Android).
    ///
    /// The icon must be placed in the app's `res/drawable` folder.
    pub fn large_icon(mut self, large_icon: impl Into<String>) -> Self {
        self.data.large_icon.replace(large_icon.into());
        self
    }

    /// Icon color on Android.
    pub fn icon_color(mut self, icon_color: impl Into<String>) -> Self {
        self.data.icon_color.replace(icon_color.into());
        self
    }

    /// Append an attachment to the notification.
    pub fn attachment(mut self, attachment: Attachment) -> Self {
        self.data.attachments.push(attachment);
        self
    }

    /// Adds an extra payload to store in the notification.
    pub fn extra(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        self.data
            .extra
            .insert(key.into(), serde_json::to_value(value).unwrap());
        self
    }

    /// If true, the notification cannot be dismissed by the user on Android.
    ///
    /// An application service must manage the dismissal of the notification.
    /// It is typically used to indicate a background task that is pending (e.g. a file download)
    /// or the user is engaged with (e.g. playing music).
    pub fn ongoing(mut self) -> Self {
        self.data.ongoing = true;
        self
    }

    /// Automatically cancel the notification when the user clicks on it.
    pub fn auto_cancel(mut self) -> Self {
        self.data.auto_cancel = true;
        self
    }

    /// Changes the notification presentation to be silent on iOS (no badge, no sound, not listed).
    pub fn silent(mut self) -> Self {
        self.data.silent = true;
        self
    }
}

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the notification APIs.
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
        .js_init_script(include_str!("init-iife.js").to_string())
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
