// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, fmt::Display};

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};

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
#[serde(rename_all = "camelCase")]
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
                Self::Year => "year",
                Self::Month => "month",
                Self::TwoWeeks => "twoWeeks",
                Self::Week => "week",
                Self::Day => "day",
                Self::Hour => "hour",
                Self::Minute => "minute",
                Self::Second => "second",
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
#[serde(rename_all = "camelCase")]
pub enum Schedule {
    #[serde(rename_all = "camelCase")]
    At {
        #[serde(
            serialize_with = "iso8601::serialize",
            deserialize_with = "time::serde::iso8601::deserialize"
        )]
        date: time::OffsetDateTime,
        #[serde(default)]
        repeating: bool,
        #[serde(default)]
        allow_while_idle: bool,
    },
    #[serde(rename_all = "camelCase")]
    Interval {
        interval: ScheduleInterval,
        #[serde(default)]
        allow_while_idle: bool,
    },
    #[serde(rename_all = "camelCase")]
    Every {
        interval: ScheduleEvery,
        count: u8,
        #[serde(default)]
        allow_while_idle: bool,
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
pub struct NotificationData {
    #[serde(default = "default_id")]
    pub(crate) id: i32,
    pub(crate) channel_id: Option<String>,
    pub(crate) title: Option<String>,
    pub(crate) body: Option<String>,
    pub(crate) schedule: Option<Schedule>,
    pub(crate) large_body: Option<String>,
    pub(crate) summary: Option<String>,
    pub(crate) action_type_id: Option<String>,
    pub(crate) group: Option<String>,
    #[serde(default)]
    pub(crate) group_summary: bool,
    pub(crate) sound: Option<String>,
    #[serde(default)]
    pub(crate) inbox_lines: Vec<String>,
    pub(crate) icon: Option<String>,
    pub(crate) large_icon: Option<String>,
    pub(crate) icon_color: Option<String>,
    #[serde(default)]
    pub(crate) attachments: Vec<Attachment>,
    #[serde(default)]
    pub(crate) extra: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub(crate) ongoing: bool,
    #[serde(default)]
    pub(crate) auto_cancel: bool,
    #[serde(default)]
    pub(crate) silent: bool,
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
            silent: false,
        }
    }
}

/// Permission state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    /// Permission access has been granted.
    Granted,
    /// Permission access has been denied.
    Denied,
    /// Unknown state. Must request permission.
    Unknown,
}

impl Display for PermissionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Granted => write!(f, "granted"),
            Self::Denied => write!(f, "denied"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

impl Serialize for PermissionState {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

impl<'de> Deserialize<'de> for PermissionState {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "granted" => Ok(Self::Granted),
            "denied" => Ok(Self::Denied),
            "prompt" => Ok(Self::Unknown),
            _ => Err(DeError::custom(format!("unknown permission state '{s}'"))),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingNotification {
    id: i32,
    title: Option<String>,
    body: Option<String>,
    schedule: Schedule,
}

impl PendingNotification {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActiveNotification {
    id: i32,
    tag: Option<String>,
    title: Option<String>,
    body: Option<String>,
    group: Option<String>,
    #[serde(default)]
    group_summary: bool,
    #[serde(default)]
    data: HashMap<String, String>,
    #[serde(default)]
    extra: HashMap<String, serde_json::Value>,
    #[serde(default)]
    attachments: Vec<Attachment>,
    action_type_id: Option<String>,
    schedule: Option<Schedule>,
    sound: Option<String>,
}

impl ActiveNotification {
    pub fn id(&self) -> i32 {
        self.id
    }

    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    pub fn group_summary(&self) -> bool {
        self.group_summary
    }

    pub fn data(&self) -> &HashMap<String, String> {
        &self.data
    }

    pub fn extra(&self) -> &HashMap<String, serde_json::Value> {
        &self.extra
    }

    pub fn attachments(&self) -> &[Attachment] {
        &self.attachments
    }

    pub fn action_type_id(&self) -> Option<&str> {
        self.action_type_id.as_deref()
    }

    pub fn schedule(&self) -> Option<&Schedule> {
        self.schedule.as_ref()
    }

    pub fn sound(&self) -> Option<&str> {
        self.sound.as_deref()
    }
}

#[cfg(mobile)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionType {
    id: String,
    actions: Vec<Action>,
    hidden_previews_body_placeholder: Option<String>,
    custom_dismiss_action: bool,
    allow_in_car_play: bool,
    hidden_previews_show_title: bool,
    hidden_previews_show_subtitle: bool,
}

#[cfg(mobile)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Action {
    id: String,
    title: String,
    requires_authentication: bool,
    foreground: bool,
    destructive: bool,
    input: bool,
    input_button_title: Option<String>,
    input_placeholder: Option<String>,
}

#[cfg(target_os = "android")]
pub use android::*;

#[cfg(target_os = "android")]
mod android {
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};

    #[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
    #[repr(u8)]
    pub enum Importance {
        None = 0,
        Min = 1,
        Low = 2,
        Default = 3,
        High = 4,
    }

    impl Default for Importance {
        fn default() -> Self {
            Self::Default
        }
    }

    #[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
    #[repr(i8)]
    pub enum Visibility {
        Secret = -1,
        Private = 0,
        Public = 1,
    }

    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct Channel {
        id: String,
        name: String,
        description: Option<String>,
        sound: Option<String>,
        lights: bool,
        light_color: Option<String>,
        vibration: bool,
        importance: Importance,
        visibility: Option<Visibility>,
    }

    #[derive(Debug)]
    pub struct ChannelBuilder(Channel);

    impl Channel {
        pub fn builder(id: impl Into<String>, name: impl Into<String>) -> ChannelBuilder {
            ChannelBuilder(Self {
                id: id.into(),
                name: name.into(),
                description: None,
                sound: None,
                lights: false,
                light_color: None,
                vibration: false,
                importance: Default::default(),
                visibility: None,
            })
        }

        pub fn id(&self) -> &str {
            &self.id
        }

        pub fn name(&self) -> &str {
            &self.name
        }

        pub fn description(&self) -> Option<&str> {
            self.description.as_deref()
        }

        pub fn sound(&self) -> Option<&str> {
            self.sound.as_deref()
        }

        pub fn lights(&self) -> bool {
            self.lights
        }

        pub fn light_color(&self) -> Option<&str> {
            self.light_color.as_deref()
        }

        pub fn vibration(&self) -> bool {
            self.vibration
        }

        pub fn importance(&self) -> Importance {
            self.importance
        }

        pub fn visibility(&self) -> Option<Visibility> {
            self.visibility
        }
    }

    impl ChannelBuilder {
        pub fn description(mut self, description: impl Into<String>) -> Self {
            self.0.description.replace(description.into());
            self
        }

        pub fn sound(mut self, sound: impl Into<String>) -> Self {
            self.0.sound.replace(sound.into());
            self
        }

        pub fn lights(mut self, lights: bool) -> Self {
            self.0.lights = lights;
            self
        }

        pub fn light_color(mut self, color: impl Into<String>) -> Self {
            self.0.light_color.replace(color.into());
            self
        }

        pub fn vibration(mut self, vibration: bool) -> Self {
            self.0.vibration = vibration;
            self
        }

        pub fn importance(mut self, importance: Importance) -> Self {
            self.0.importance = importance;
            self
        }

        pub fn visibility(mut self, visibility: Visibility) -> Self {
            self.0.visibility.replace(visibility);
            self
        }

        pub fn build(self) -> Channel {
            self.0
        }
    }
}
