// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, fmt::Display};

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};
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
use url::Url;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    id: String,
    url: Url,
}

impl Attachment {
    pub fn new(id: impl Into<String>, url: Url) -> Self {
        Self { id: id.into(), url }
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ScheduleInterval {
    pub year: Option<u8>,
    pub month: Option<u8>,
    pub day: Option<u8>,
    pub weekday: Option<u8>,
    pub hour: Option<u8>,
    pub minute: Option<u8>,
    pub second: Option<u8>,
}

#[derive(Debug)]
pub enum ScheduleEvery {
    Year,
    Month,
    TwoWeeks,
    Week,
    Day,
    Hour,
    Minute,
    Second,
}

impl Display for ScheduleEvery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Year => "Year",
                Self::Month => "Month",
                Self::TwoWeeks => "TwoWeeks",
                Self::Week => "Week",
                Self::Day => "Day",
                Self::Hour => "Hour",
                Self::Minute => "Minute",
                Self::Second => "Second",
            }
        )
    }
}

impl Serialize for ScheduleEvery {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl<'de> Deserialize<'de> for ScheduleEvery {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "year" => Ok(Self::Year),
            "month" => Ok(Self::Month),
            "twoweeks" => Ok(Self::TwoWeeks),
            "week" => Ok(Self::Week),
            "day" => Ok(Self::Day),
            "hour" => Ok(Self::Hour),
            "minute" => Ok(Self::Minute),
            "second" => Ok(Self::Second),
            _ => Err(DeError::custom(format!("unknown every kind '{s}'"))),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum Schedule {
    At {
        #[serde(
            serialize_with = "iso8601::serialize",
            deserialize_with = "time::serde::iso8601::deserialize"
        )]
        date: time::OffsetDateTime,
        #[serde(default)]
        repeating: bool,
    },
    Interval(ScheduleInterval),
    Every {
        interval: ScheduleEvery,
    },
}

// custom ISO-8601 serialization that does not use 6 digits for years.
mod iso8601 {
    use serde::{ser::Error as _, Serialize, Serializer};
    use time::{
        format_description::well_known::iso8601::{Config, EncodedConfig},
        format_description::well_known::Iso8601,
        OffsetDateTime,
    };

    const SERDE_CONFIG: EncodedConfig = Config::DEFAULT.encode();

    pub fn serialize<S: Serializer>(
        datetime: &OffsetDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        datetime
            .format(&Iso8601::<SERDE_CONFIG>)
            .map_err(S::Error::custom)?
            .serialize(serializer)
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NotificationData {
    /// Notification id.
    #[serde(default = "default_id")]
    id: i32,
    channel_id: Option<String>,
    title: Option<String>,
    body: Option<String>,
    schedule: Option<Schedule>,
    large_body: Option<String>,
    summary: Option<String>,
    action_type_id: Option<String>,
    group: Option<String>,
    #[serde(default)]
    group_summary: bool,
    sound: Option<String>,
    #[serde(default)]
    inbox_lines: Vec<String>,
    icon: Option<String>,
    large_icon: Option<String>,
    icon_color: Option<String>,
    #[serde(default)]
    attachments: Vec<Attachment>,
    #[serde(default)]
    extra: HashMap<String, serde_json::Value>,
    #[serde(default)]
    ongoing: bool,
    #[serde(default)]
    auto_cancel: bool,
}

fn default_id() -> i32 {
    rand::random()
}

impl Default for NotificationData {
    fn default() -> Self {
        Self {
            id: default_id(),
            channel_id: None,
            title: None,
            body: None,
            schedule: None,
            large_body: None,
            summary: None,
            action_type_id: None,
            group: None,
            group_summary: false,
            sound: None,
            inbox_lines: Vec::new(),
            icon: None,
            large_icon: None,
            icon_color: None,
            attachments: Vec::new(),
            extra: Default::default(),
            ongoing: false,
            auto_cancel: false,
        }
    }
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
