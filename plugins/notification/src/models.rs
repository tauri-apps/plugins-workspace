// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, fmt::Display};

use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize, Serializer};

use url::Url;

/// A media file attached to a notification.
///
/// Attachments are only used on mobile; desktop notifications ignore them.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Attachment {
    id: String,
    url: Url,
}

impl Attachment {
    /// Creates a new attachment with the given identifier and URL.
    ///
    /// The URL accepts the `asset` and `file` protocols.
    pub fn new(id: impl Into<String>, url: Url) -> Self {
        Self { id: id.into(), url }
    }
}

/// The set of date fields a notification must match to be delivered.
///
/// Fields left as [`None`] match any value, so the notification fires on every date
/// whose remaining components match. Used by [`Schedule::Interval`].
#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleInterval {
    /// The year the notification fires on.
    pub year: Option<u8>,
    /// The month of the year the notification fires on.
    pub month: Option<u8>,
    /// The day of the month the notification fires on.
    pub day: Option<u8>,
    /// The day of the week the notification fires on.
    ///
    /// 1 - Sunday, 2 - Monday, 3 - Tuesday, 4 - Wednesday, 5 - Thursday, 6 - Friday, 7 - Saturday.
    pub weekday: Option<u8>,
    /// The hour of the day the notification fires on, in the 24-hour clock.
    pub hour: Option<u8>,
    /// The minute of the hour the notification fires on.
    pub minute: Option<u8>,
    /// The second of the minute the notification fires on.
    pub second: Option<u8>,
}

/// The unit of the repeating interval used by [`Schedule::Every`].
///
/// It is serialized as its lowercase camelCase name, e.g. `twoWeeks`.
#[derive(Debug)]
pub enum ScheduleEvery {
    /// Repeats every year.
    ///
    /// On Android a year is approximated as 52 weeks.
    Year,
    /// Repeats every month.
    ///
    /// On Android a month is approximated as 30 days.
    Month,
    /// Repeats every two weeks.
    TwoWeeks,
    /// Repeats every week.
    Week,
    /// Repeats every day.
    Day,
    /// Repeats every hour.
    Hour,
    /// Repeats every minute.
    Minute,
    /// Repeats every second.
    ///
    /// Not supported on iOS, where repeating triggers must be at least a minute apart.
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

/// Defines when a notification is delivered.
///
/// Scheduling is only implemented on mobile; the desktop implementation delivers the
/// notification immediately and ignores the schedule.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Schedule {
    /// Fires at a specific date and time, which must be in the future.
    #[serde(rename_all = "camelCase")]
    At {
        /// The date and time the notification fires at, serialized as an ISO-8601 string.
        #[serde(
            serialize_with = "iso8601::serialize",
            deserialize_with = "time::serde::iso8601::deserialize"
        )]
        date: time::OffsetDateTime,
        /// Whether the notification keeps repeating, using the duration between the moment it is
        /// scheduled and `date` as the interval. Defaults to `false`.
        ///
        /// The interval must be at least one minute on iOS.
        #[serde(default)]
        repeating: bool,
        /// Whether the notification is allowed to fire while the device is in low-power idle
        /// (Doze) mode. Defaults to `false`.
        ///
        /// Only used on Android.
        #[serde(default)]
        allow_while_idle: bool,
    },
    /// Fires whenever the current date matches every field set on the given interval.
    #[serde(rename_all = "camelCase")]
    Interval {
        /// The date fields the current date must match for the notification to fire.
        interval: ScheduleInterval,
        /// Whether the notification is allowed to fire while the device is in low-power idle
        /// (Doze) mode. Defaults to `false`.
        ///
        /// Only used on Android.
        #[serde(default)]
        allow_while_idle: bool,
    },
    /// Fires repeatedly, once every `count` times the given interval unit.
    #[serde(rename_all = "camelCase")]
    Every {
        /// The unit of the repeating interval.
        interval: ScheduleEvery,
        /// How many interval units elapse between each notification.
        count: u8,
        /// Whether the notification is allowed to fire while the device is in low-power idle
        /// (Doze) mode. Defaults to `false`.
        ///
        /// Only used on Android.
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

/// The payload of a notification, as sent to the platform implementation.
///
/// Build it with [`NotificationBuilder`](crate::NotificationBuilder) rather than constructing it directly.
/// The identifier defaults to a random 32-bit integer when it is not provided.
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
    /// Set the time that the event occurred (milliseconds since epoch).
    /// Notifications in the panel are sorted by this time.
    /// Android only.
    pub(crate) when: Option<i64>,
    /// Show the `when` field as a stopwatch (elapsed time).
    /// Android only.
    #[serde(default)]
    pub(crate) uses_chronometer: bool,
    /// Sets the Chronometer to count down instead of counting up.
    /// Only relevant if `uses_chronometer` is true.
    /// Android only (API 24+).
    #[serde(default)]
    pub(crate) chronometer_count_down: bool,
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
            when: None,
            uses_chronometer: false,
            chronometer_count_down: false,
        }
    }
}

/// A notification that was scheduled and has not been delivered yet.
///
/// Returned by `Notification::pending`, which is only available on mobile.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingNotification {
    id: i32,
    title: Option<String>,
    body: Option<String>,
    schedule: Schedule,
}

impl PendingNotification {
    /// The notification identifier.
    pub fn id(&self) -> i32 {
        self.id
    }

    /// The notification title, if it was set.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// The notification body, if it was set.
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    /// The schedule that determines when the notification is delivered.
    pub fn schedule(&self) -> &Schedule {
        &self.schedule
    }
}

/// A notification that was delivered and is still visible in the notification center.
///
/// Returned by `Notification::active`, which is only available on mobile.
/// Which fields are populated depends on the platform, since Android and iOS expose
/// different information about delivered notifications.
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
    /// The notification identifier.
    pub fn id(&self) -> i32 {
        self.id
    }

    /// The tag the notification was posted with.
    ///
    /// Only set on Android.
    pub fn tag(&self) -> Option<&str> {
        self.tag.as_deref()
    }

    /// The notification title, if it was set.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// The notification body, if it was set.
    pub fn body(&self) -> Option<&str> {
        self.body.as_deref()
    }

    /// The identifier of the group the notification belongs to.
    ///
    /// Only set on Android.
    pub fn group(&self) -> Option<&str> {
        self.group.as_deref()
    }

    /// Whether the notification is the summary of its group.
    ///
    /// Only set on Android. Defaults to `false`.
    pub fn group_summary(&self) -> bool {
        self.group_summary
    }

    /// The platform extras attached to the notification, as string values.
    ///
    /// Only set on Android, where it holds the `android.app.Notification` extras bundle.
    pub fn data(&self) -> &HashMap<String, String> {
        &self.data
    }

    /// The extra payload that was stored in the notification.
    pub fn extra(&self) -> &HashMap<String, serde_json::Value> {
        &self.extra
    }

    /// The attachments of the notification.
    ///
    /// Only set on iOS.
    pub fn attachments(&self) -> &[Attachment] {
        &self.attachments
    }

    /// The identifier of the action type the notification was registered with.
    ///
    /// Only set on iOS.
    pub fn action_type_id(&self) -> Option<&str> {
        self.action_type_id.as_deref()
    }

    /// The schedule the notification was delivered with, if it was scheduled.
    pub fn schedule(&self) -> Option<&Schedule> {
        self.schedule.as_ref()
    }

    /// The sound resource name of the notification.
    ///
    /// Only set on iOS.
    pub fn sound(&self) -> Option<&str> {
        self.sound.as_deref()
    }
}

/// A group of [`Action`]s a notification can display, referenced by
/// [`NotificationBuilder::action_type_id`](crate::NotificationBuilder::action_type_id).
///
/// Register it with `Notification::register_action_types` before sending a notification that uses it.
/// It maps to a `UNNotificationCategory` on iOS and to an action group on Android.
///
/// Only available on mobile. Use [`ActionType::builder`] to construct one.
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

/// Builder for an [`ActionType`], created with [`ActionType::builder`].
///
/// Only available on mobile.
#[cfg(mobile)]
#[derive(Debug)]
pub struct ActionTypeBuilder(ActionType);

#[cfg(mobile)]
impl ActionType {
    /// Creates a builder for an action type with the given identifier.
    ///
    /// All the optional settings default to `false` or [`None`];
    /// call [`ActionTypeBuilder::build`] to get the [`ActionType`].
    pub fn builder(id: impl Into<String>) -> ActionTypeBuilder {
        ActionTypeBuilder(Self {
            id: id.into(),
            actions: Vec::new(),
            hidden_previews_body_placeholder: None,
            custom_dismiss_action: false,
            allow_in_car_play: false,
            hidden_previews_show_title: false,
            hidden_previews_show_subtitle: false,
        })
    }

    /// The identifier of this action type.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The actions associated with this action type.
    pub fn actions(&self) -> &[Action] {
        &self.actions
    }

    /// The placeholder shown instead of the notification body when previews are hidden.
    ///
    /// Only used on iOS.
    pub fn hidden_previews_body_placeholder(&self) -> Option<&str> {
        self.hidden_previews_body_placeholder.as_deref()
    }

    /// Whether the app is notified when the user dismisses the notification.
    ///
    /// Only used on iOS.
    pub fn custom_dismiss_action(&self) -> bool {
        self.custom_dismiss_action
    }

    /// Whether the notification can be displayed in a CarPlay environment.
    ///
    /// Only used on iOS.
    pub fn allow_in_car_play(&self) -> bool {
        self.allow_in_car_play
    }

    /// Whether the notification title is shown even when previews are hidden.
    ///
    /// Only used on iOS.
    pub fn hidden_previews_show_title(&self) -> bool {
        self.hidden_previews_show_title
    }

    /// Whether the notification subtitle is shown even when previews are hidden.
    ///
    /// Only used on iOS.
    pub fn hidden_previews_show_subtitle(&self) -> bool {
        self.hidden_previews_show_subtitle
    }
}

#[cfg(mobile)]
impl ActionTypeBuilder {
    /// Sets the actions associated with this action type.
    pub fn actions(mut self, actions: Vec<Action>) -> Self {
        self.0.actions = actions;
        self
    }

    /// Sets the placeholder shown instead of the notification body when previews are hidden.
    ///
    /// Only used on iOS.
    pub fn hidden_previews_body_placeholder(
        mut self,
        hidden_previews_body_placeholder: impl Into<String>,
    ) -> Self {
        self.0
            .hidden_previews_body_placeholder
            .replace(hidden_previews_body_placeholder.into());
        self
    }

    /// Sets whether the app is notified when the user dismisses the notification.
    ///
    /// Only used on iOS.
    pub fn custom_dismiss_action(mut self, custom_dismiss_action: bool) -> Self {
        self.0.custom_dismiss_action = custom_dismiss_action;
        self
    }

    /// Sets whether the notification can be displayed in a CarPlay environment.
    ///
    /// Only used on iOS.
    pub fn allow_in_car_play(mut self, allow_in_car_play: bool) -> Self {
        self.0.allow_in_car_play = allow_in_car_play;
        self
    }

    /// Sets whether the notification title is shown even when previews are hidden.
    ///
    /// Only used on iOS.
    pub fn hidden_previews_show_title(mut self, hidden_previews_show_title: bool) -> Self {
        self.0.hidden_previews_show_title = hidden_previews_show_title;
        self
    }

    /// Sets whether the notification subtitle is shown even when previews are hidden.
    ///
    /// Only used on iOS.
    pub fn hidden_previews_show_subtitle(mut self, hidden_previews_show_subtitle: bool) -> Self {
        self.0.hidden_previews_show_subtitle = hidden_previews_show_subtitle;
        self
    }

    /// Builds the [`ActionType`].
    pub fn build(self) -> ActionType {
        self.0
    }
}

/// A button the user can tap on a notification, belonging to an [`ActionType`].
///
/// It maps to a `UNNotificationAction` on iOS. On Android only the identifier, the title
/// and the input flag are used.
///
/// Only available on mobile. Use [`Action::builder`] to construct one.
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

/// Builder for an [`Action`], created with [`Action::builder`].
///
/// Only available on mobile.
#[cfg(mobile)]
#[derive(Debug)]
pub struct ActionBuilder(Action);

#[cfg(mobile)]
impl Action {
    /// Creates a builder for an action with the given identifier and button title.
    ///
    /// All the optional settings default to `false` or [`None`];
    /// call [`ActionBuilder::build`] to get the [`Action`].
    pub fn builder(id: impl Into<String>, title: impl Into<String>) -> ActionBuilder {
        ActionBuilder(Self {
            id: id.into(),
            title: title.into(),
            requires_authentication: false,
            foreground: false,
            destructive: false,
            input: false,
            input_button_title: None,
            input_placeholder: None,
        })
    }

    /// The identifier of this action, reported back when the user triggers it.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// The text displayed on the action button.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Whether the device must be unlocked for the action to run.
    ///
    /// Only used on iOS.
    pub fn requires_authentication(&self) -> bool {
        self.requires_authentication
    }

    /// Whether the app is brought to the foreground when the action is triggered.
    ///
    /// Only used on iOS.
    pub fn foreground(&self) -> bool {
        self.foreground
    }

    /// Whether the action is displayed as destructive, usually in red.
    ///
    /// Only used on iOS.
    pub fn destructive(&self) -> bool {
        self.destructive
    }

    /// Whether triggering the action lets the user type a text response.
    pub fn input(&self) -> bool {
        self.input
    }

    /// The text displayed on the button that submits the text input.
    ///
    /// Only used on iOS.
    pub fn input_button_title(&self) -> Option<&str> {
        self.input_button_title.as_deref()
    }

    /// The placeholder displayed on the empty text input field.
    ///
    /// Only used on iOS.
    pub fn input_placeholder(&self) -> Option<&str> {
        self.input_placeholder.as_deref()
    }
}

#[cfg(mobile)]
impl ActionBuilder {
    /// Sets whether the device must be unlocked for the action to run.
    ///
    /// Only used on iOS.
    pub fn requires_authentication(mut self, requires_authentication: bool) -> Self {
        self.0.requires_authentication = requires_authentication;
        self
    }

    /// Sets whether the app is brought to the foreground when the action is triggered.
    ///
    /// Only used on iOS.
    pub fn foreground(mut self, foreground: bool) -> Self {
        self.0.foreground = foreground;
        self
    }

    /// Sets whether the action is displayed as destructive, usually in red.
    ///
    /// Only used on iOS.
    pub fn destructive(mut self, destructive: bool) -> Self {
        self.0.destructive = destructive;
        self
    }

    /// Sets whether triggering the action lets the user type a text response.
    pub fn input(mut self, input: bool) -> Self {
        self.0.input = input;
        self
    }

    /// Sets the text displayed on the button that submits the text input.
    ///
    /// Only used on iOS.
    pub fn input_button_title(mut self, input_button_title: impl Into<String>) -> Self {
        self.0.input_button_title.replace(input_button_title.into());
        self
    }

    /// Sets the placeholder displayed on the empty text input field.
    ///
    /// Only used on iOS.
    pub fn input_placeholder(mut self, input_placeholder: impl Into<String>) -> Self {
        self.0.input_placeholder.replace(input_placeholder.into());
        self
    }

    /// Builds the [`Action`].
    pub fn build(self) -> Action {
        self.0
    }
}

#[cfg(target_os = "android")]
pub use android::*;

#[cfg(target_os = "android")]
mod android {
    use serde::{Deserialize, Serialize};
    use serde_repr::{Deserialize_repr, Serialize_repr};

    /// How much the notifications of a [`Channel`] interrupt the user.
    ///
    /// It maps to the `NotificationManager.IMPORTANCE_*` constants and is serialized as its
    /// integer value. Only available on Android.
    #[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
    #[repr(u8)]
    pub enum Importance {
        /// The notifications are not shown.
        None = 0,
        /// The notifications are only shown in the shade, below the fold, without a status bar icon.
        Min = 1,
        /// The notifications are shown without a sound.
        Low = 2,
        /// The notifications are shown and make a sound.
        ///
        /// This is the value used when the channel does not define an importance.
        Default = 3,
        /// The notifications are shown, make a sound and pop up as a heads-up notification.
        High = 4,
    }

    impl Default for Importance {
        fn default() -> Self {
            Self::Default
        }
    }

    /// How much of a notification is shown on the lock screen.
    ///
    /// It maps to the `Notification.VISIBILITY_*` constants and is serialized as its
    /// integer value. Only available on Android.
    #[derive(Debug, Clone, Copy, Serialize_repr, Deserialize_repr)]
    #[repr(i8)]
    pub enum Visibility {
        /// The notification is not shown on the lock screen at all.
        Secret = -1,
        /// The notification is shown on the lock screen with its sensitive content hidden.
        ///
        /// This is the value used when the channel does not define a visibility.
        Private = 0,
        /// The notification is shown in full on the lock screen.
        Public = 1,
    }

    /// A notification channel, the category users configure notification behavior on.
    ///
    /// Notifications reference a channel through
    /// [`NotificationBuilder::channel_id`](crate::NotificationBuilder::channel_id) and are not
    /// delivered when the channel does not exist. Only available on Android.
    /// Use [`Channel::builder`] to construct one.
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

    /// Builder for a [`Channel`], created with [`Channel::builder`].
    ///
    /// Only available on Android.
    #[derive(Debug)]
    pub struct ChannelBuilder(Channel);

    impl Channel {
        /// Creates a builder for a channel with the given identifier and user visible name.
        ///
        /// Lights and vibration are disabled, the importance defaults to [`Importance::Default`]
        /// and the remaining settings default to [`None`];
        /// call [`ChannelBuilder::build`] to get the [`Channel`].
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

        /// The identifier of this channel.
        pub fn id(&self) -> &str {
            &self.id
        }

        /// The user visible name of this channel.
        pub fn name(&self) -> &str {
            &self.name
        }

        /// The user visible description of this channel.
        pub fn description(&self) -> Option<&str> {
            self.description.as_deref()
        }

        /// The name of the sound resource played by the notifications of this channel.
        ///
        /// The resource must be placed in the app's `res/raw` folder.
        pub fn sound(&self) -> Option<&str> {
            self.sound.as_deref()
        }

        /// Whether the notifications of this channel blink the device light.
        pub fn lights(&self) -> bool {
            self.lights
        }

        /// The color of the device light, as a color string such as `#ff0000`.
        pub fn light_color(&self) -> Option<&str> {
            self.light_color.as_deref()
        }

        /// Whether the notifications of this channel vibrate the device.
        pub fn vibration(&self) -> bool {
            self.vibration
        }

        /// How much the notifications of this channel interrupt the user.
        pub fn importance(&self) -> Importance {
            self.importance
        }

        /// How much of the notifications of this channel is shown on the lock screen.
        ///
        /// [`Visibility::Private`] is used when this is [`None`].
        pub fn visibility(&self) -> Option<Visibility> {
            self.visibility
        }
    }

    impl ChannelBuilder {
        /// Sets the user visible description of the channel.
        pub fn description(mut self, description: impl Into<String>) -> Self {
            self.0.description.replace(description.into());
            self
        }

        /// Sets the name of the sound resource played by the notifications of this channel.
        ///
        /// The resource must be placed in the app's `res/raw` folder.
        pub fn sound(mut self, sound: impl Into<String>) -> Self {
            self.0.sound.replace(sound.into());
            self
        }

        /// Sets whether the notifications of this channel blink the device light.
        pub fn lights(mut self, lights: bool) -> Self {
            self.0.lights = lights;
            self
        }

        /// Sets the color of the device light, as a color string such as `#ff0000`.
        pub fn light_color(mut self, color: impl Into<String>) -> Self {
            self.0.light_color.replace(color.into());
            self
        }

        /// Sets whether the notifications of this channel vibrate the device.
        pub fn vibration(mut self, vibration: bool) -> Self {
            self.0.vibration = vibration;
            self
        }

        /// Sets how much the notifications of this channel interrupt the user.
        pub fn importance(mut self, importance: Importance) -> Self {
            self.0.importance = importance;
            self
        }

        /// Sets how much of the notifications of this channel is shown on the lock screen.
        pub fn visibility(mut self, visibility: Visibility) -> Self {
            self.0.visibility.replace(visibility);
            self
        }

        /// Builds the [`Channel`].
        pub fn build(self) -> Channel {
            self.0
        }
    }
}
