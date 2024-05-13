#[cfg(all(unix, not(target_os = "macos")))]
use super::{
    hints::{CustomHintType, Hint},
    urgency::Urgency,
    xdg,
};

#[cfg(all(unix, not(target_os = "macos"), feature = "images"))]
use super::image::Image;

#[cfg(all(unix, target_os = "macos"))]
use super::macos;
#[cfg(target_os = "windows")]
use super::windows;

use super::{error::*, timeout::Timeout};

#[cfg(all(unix, not(target_os = "macos")))]
use std::collections::{HashMap, HashSet};

// Returns the name of the current executable, used as a default for `Notification.appname`.
fn exe_name() -> String {
    std::env::current_exe()
        .unwrap()
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .to_owned()
}

/// Desktop notification.
///
/// A desktop notification is configured via builder pattern, before it is launched with `show()`.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Notification {
    /// Filled by default with executable name.
    pub appname: String,

    /// Single line to summarize the content.
    pub summary: String,

    /// Subtitle for macOS
    pub subtitle: Option<String>,

    /// Multiple lines possible, may support simple markup,
    /// check out `get_capabilities()` -> `body-markup` and `body-hyperlinks`.
    pub body: String,

    /// Use a file:// URI or a name in an icon theme, must be compliant freedesktop.org.
    pub icon: String,

    /// Check out `Hint`
    ///
    /// # warning
    /// this does not hold all hints, [`Hint::Custom`] and [`Hint::CustomInt`] are held elsewhere,
    // /// please access hints via [`Notification::get_hints`].
    #[cfg(all(unix, not(target_os = "macos")))]
    pub hints: HashSet<Hint>,

    #[cfg(all(unix, not(target_os = "macos")))]
    pub(crate) hints_unique: HashMap<(String, CustomHintType), Hint>,

    /// See `Notification::actions()` and `Notification::action()`
    pub actions: Vec<String>,

    #[cfg(target_os = "macos")]
    pub(crate) sound_name: Option<String>,

    #[cfg(target_os = "windows")]
    pub(crate) sound_name: Option<String>,

    #[cfg(target_os = "windows")]
    pub(crate) path_to_image: Option<String>,

    #[cfg(target_os = "windows")]
    pub(crate) app_id: Option<String>,

    #[cfg(all(unix, not(target_os = "macos")))]
    pub(crate) bus: xdg::NotificationBus,

    /// Lifetime of the Notification in ms. Often not respected by server, sorry.
    pub timeout: Timeout, // both gnome and galago want allow for -1

    /// Only to be used on the receive end. Use Notification hand for updating.
    pub(crate) id: Option<u32>,
}

impl Notification {
    /// Constructs a new Notification.
    ///
    /// Most fields are empty by default, only `appname` is initialized with the name of the current
    /// executable.
    /// The appname is used by some desktop environments to group notifications.
    pub fn new() -> Notification {
        Notification::default()
    }

    /// This is for testing purposes only and will not work with actual implementations.
    #[cfg(all(unix, not(target_os = "macos")))]
    #[doc(hidden)]
    #[deprecated(note = "this is a test only feature")]
    pub fn at_bus(sub_bus: &str) -> Notification {
        let bus = xdg::NotificationBus::custom(sub_bus)
            .ok_or("invalid subpath")
            .unwrap();
        Notification {
            bus,
            ..Notification::default()
        }
    }

    /// Overwrite the appname field used for Notification.
    ///
    /// # Platform Support
    /// Please note that this method has no effect on macOS. Here you can only set the application via [`set_application()`](fn.set_application.html)
    pub fn appname(&mut self, appname: &str) -> &mut Notification {
        appname.clone_into(&mut self.appname);
        self
    }

    /// Set the `summary`.
    ///
    /// Often acts as title of the notification. For more elaborate content use the `body` field.
    pub fn summary(&mut self, summary: &str) -> &mut Notification {
        summary.clone_into(&mut self.summary);
        self
    }

    /// Set the `subtitle`.
    ///
    /// This is only useful on macOS, it's not part of the XDG specification and will therefore be eaten by gremlins under your CPU 😈🤘.
    pub fn subtitle(&mut self, subtitle: &str) -> &mut Notification {
        self.subtitle = Some(subtitle.to_owned());
        self
    }

    /// Manual wrapper for `Hint::ImageData`
    #[cfg(all(feature = "images", unix, not(target_os = "macos")))]
    pub fn image_data(&mut self, image: Image) -> &mut Notification {
        self.hint(Hint::ImageData(image));
        self
    }

    /// Wrapper for `Hint::ImagePath`
    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn image_path(&mut self, path: &str) -> &mut Notification {
        self.hint(Hint::ImagePath(path.to_string()));
        self
    }

    /// Wrapper for `NotificationHint::ImagePath`
    #[cfg(target_os = "windows")]
    pub fn image_path(&mut self, path: &str) -> &mut Notification {
        self.path_to_image = Some(path.to_string());
        self
    }

    /// app's System.AppUserModel.ID
    #[cfg(target_os = "windows")]
    pub fn app_id(&mut self, app_id: &str) -> &mut Notification {
        self.app_id = Some(app_id.to_string());
        self
    }

    /// Wrapper for `Hint::ImageData`
    #[cfg(all(feature = "images", unix, not(target_os = "macos")))]
    pub fn image<T: AsRef<std::path::Path> + Sized>(
        &mut self,
        path: T,
    ) -> Result<&mut Notification> {
        let img = Image::open(&path)?;
        self.hint(Hint::ImageData(img));
        Ok(self)
    }

    /// Wrapper for `Hint::SoundName`
    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn sound_name(&mut self, name: &str) -> &mut Notification {
        self.hint(Hint::SoundName(name.to_owned()));
        self
    }

    /// Set the `sound_name` for the `NSUserNotification`
    #[cfg(any(target_os = "macos", target_os = "windows"))]
    pub fn sound_name(&mut self, name: &str) -> &mut Notification {
        self.sound_name = Some(name.to_owned());
        self
    }

    /// Set the content of the `body` field.
    ///
    /// Multiline textual content of the notification.
    /// Each line should be treated as a paragraph.
    /// Simple html markup should be supported, depending on the server implementation.
    pub fn body(&mut self, body: &str) -> &mut Notification {
        body.clone_into(&mut self.body);
        self
    }

    /// Set the `icon` field.
    ///
    /// You can use common icon names here, usually those in `/usr/share/icons`
    /// can all be used.
    /// You can also use an absolute path to file.
    ///
    /// # Platform support
    /// macOS does not have support manually setting the icon. However you can pretend to be another app using [`set_application()`](fn.set_application.html)
    pub fn icon(&mut self, icon: &str) -> &mut Notification {
        icon.clone_into(&mut self.icon);
        self
    }

    /// Set the `icon` field automatically.
    ///
    /// This looks at your binary's name and uses it to set the icon.
    ///
    /// # Platform support
    /// macOS does not support manually setting the icon. However you can pretend to be another app using [`set_application()`](fn.set_application.html)
    pub fn auto_icon(&mut self) -> &mut Notification {
        self.icon = exe_name();
        self
    }

    /// Adds a hint.
    ///
    /// This method will add a hint to the internal hint [`HashSet`].
    /// Hints must be of type [`Hint`].
    ///
    /// Many of these are again wrapped by more convenient functions such as:
    ///
    /// * `sound_name(...)`
    /// * `urgency(...)`
    /// * [`image(...)`](#method.image) or
    ///   * [`image_data(...)`](#method.image_data)
    ///   * [`image_path(...)`](#method.image_path)
    ///
    /// # Platform support
    /// Most of these hints don't even have an effect on the big XDG Desktops, they are completely tossed on macOS.
    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn hint(&mut self, hint: Hint) -> &mut Notification {
        match hint {
            Hint::CustomInt(k, v) => {
                self.hints_unique
                    .insert((k.clone(), CustomHintType::Int), Hint::CustomInt(k, v));
            }
            Hint::Custom(k, v) => {
                self.hints_unique
                    .insert((k.clone(), CustomHintType::String), Hint::Custom(k, v));
            }
            _ => {
                self.hints.insert(hint);
            }
        }
        self
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    pub(crate) fn get_hints(&self) -> impl Iterator<Item = &Hint> {
        self.hints.iter().chain(self.hints_unique.values())
    }

    /// Set the `timeout`.
    ///
    /// Accepts multiple types that implement `Into<Timeout>`.
    ///
    /// ## `i31`
    ///
    /// This sets the time (in milliseconds) from the time the notification is displayed until it is
    /// closed again by the Notification Server.
    /// According to [specification](https://developer.gnome.org/notification-spec/)
    /// -1 will leave the timeout to be set by the server and
    /// 0 will cause the notification never to expire.

    /// ## [Duration](`std::time::Duration`)
    ///
    /// When passing a [`Duration`](`std::time::Duration`) we will try convert it into milliseconds.
    ///
    /// # Platform support
    /// This only works on XDG Desktops, macOS does not support manually setting the timeout.
    pub fn timeout<T: Into<Timeout>>(&mut self, timeout: T) -> &mut Notification {
        self.timeout = timeout.into();
        self
    }

    /// Set the `urgency`.
    ///
    /// Pick between Medium, Low and High.
    ///
    /// # Platform support
    /// Most Desktops on linux and bsd are far too relaxed to pay any attention to this.
    /// In macOS this does not exist
    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn urgency(&mut self, urgency: Urgency) -> &mut Notification {
        self.hint(Hint::Urgency(urgency)); // TODO impl as T where T: Into<Urgency>
        self
    }

    /// Set `actions`.
    ///
    /// To quote <http://www.galago-project.org/specs/notification/0.9/x408.html#command-notify>
    ///
    /// >  Actions are sent over as a list of pairs.
    /// >  Each even element in the list (starting at index 0) represents the identifier for the action.
    /// >  Each odd element in the list is the localized string that will be displayed to the user.y
    ///
    /// There is nothing fancy going on here yet.
    /// **Careful! This replaces the internal list of actions!**
    ///
    /// (xdg only)
    #[deprecated(note = "please use .action() only")]
    pub fn actions(&mut self, actions: Vec<String>) -> &mut Notification {
        self.actions = actions;
        self
    }

    /// Add an action.
    ///
    /// This adds a single action to the internal list of actions.
    ///
    /// (xdg only)
    pub fn action(&mut self, identifier: &str, label: &str) -> &mut Notification {
        self.actions.push(identifier.to_owned());
        self.actions.push(label.to_owned());
        self
    }

    /// Set an Id ahead of time
    ///
    /// Setting the id ahead of time allows overriding a known other notification.
    /// Though if you want to update a notification, it is easier to use the `update()` method of
    /// the `NotificationHandle` object that `show()` returns.
    ///
    /// (xdg only)
    pub fn id(&mut self, id: u32) -> &mut Notification {
        self.id = Some(id);
        self
    }

    /// Finalizes a Notification.
    ///
    /// Part of the builder pattern, returns a complete copy of the built notification.
    pub fn finalize(&self) -> Notification {
        self.clone()
    }

    /// Schedules a Notification
    ///
    /// Sends a Notification at the specified date.
    #[cfg(all(target_os = "macos", feature = "chrono"))]
    pub fn schedule<T: chrono::TimeZone>(
        &self,
        delivery_date: chrono::DateTime<T>,
    ) -> Result<macos::NotificationHandle> {
        macos::schedule_notification(self, delivery_date.timestamp() as f64)
    }

    /// Schedules a Notification
    ///
    /// Sends a Notification at the specified timestamp.
    /// This is a raw `f64`, if that is a bit too raw for you please activate the feature `"chrono"`,
    /// then you can use `Notification::schedule()` instead, which accepts a `chrono::DateTime<T>`.
    #[cfg(target_os = "macos")]
    pub fn schedule_raw(&self, timestamp: f64) -> Result<macos::NotificationHandle> {
        macos::schedule_notification(self, timestamp)
    }

    /// Sends Notification to D-Bus.
    ///
    /// Returns a handle to a notification
    #[cfg(all(unix, not(target_os = "macos")))]
    pub fn show(&self) -> Result<xdg::NotificationHandle> {
        xdg::show_notification(self)
    }

    /// Sends Notification to D-Bus.
    ///
    /// Returns a handle to a notification
    #[cfg(all(unix, not(target_os = "macos")))]
    #[cfg(all(feature = "async", feature = "zbus"))]
    pub async fn show_async(&self) -> Result<xdg::NotificationHandle> {
        xdg::show_notification_async(self).await
    }

    /// Sends Notification to D-Bus.
    ///
    /// Returns a handle to a notification
    #[cfg(all(unix, not(target_os = "macos")))]
    #[cfg(feature = "async")]
    // #[cfg(test)]
    pub async fn show_async_at_bus(&self, sub_bus: &str) -> Result<xdg::NotificationHandle> {
        let bus = xdg::NotificationBus::custom(sub_bus).ok_or("invalid subpath")?;
        xdg::show_notification_async_at_bus(self, bus).await
    }

    /// Sends Notification to `NSUserNotificationCenter`.
    ///
    /// Returns an `Ok` no matter what, since there is currently no way of telling the success of
    /// the notification.
    #[cfg(target_os = "macos")]
    pub fn show(&self) -> Result<macos::NotificationHandle> {
        macos::show_notification(self)
    }

    /// Sends Notification to `NSUserNotificationCenter`.
    ///
    /// Returns an `Ok` no matter what, since there is currently no way of telling the success of
    /// the notification.
    #[cfg(target_os = "windows")]
    pub fn show(&self) -> Result<()> {
        windows::show_notification(self)
    }

    /// Wraps `show()` but prints notification to stdout.
    #[cfg(all(unix, not(target_os = "macos")))]
    #[deprecated = "this was never meant to be public API"]
    pub fn show_debug(&mut self) -> Result<xdg::NotificationHandle> {
        println!(
            "Notification:\n{appname}: ({icon}) {summary:?} {body:?}\nhints: [{hints:?}]\n",
            appname = self.appname,
            summary = self.summary,
            body = self.body,
            hints = self.hints,
            icon = self.icon,
        );
        self.show()
    }
}

impl Default for Notification {
    #[cfg(all(unix, not(target_os = "macos")))]
    fn default() -> Notification {
        Notification {
            appname: exe_name(),
            summary: String::new(),
            subtitle: None,
            body: String::new(),
            icon: String::new(),
            hints: HashSet::new(),
            hints_unique: HashMap::new(),
            actions: Vec::new(),
            timeout: Timeout::Default,
            bus: Default::default(),
            id: None,
        }
    }

    #[cfg(target_os = "macos")]
    fn default() -> Notification {
        Notification {
            appname: exe_name(),
            summary: String::new(),
            subtitle: None,
            body: String::new(),
            icon: String::new(),
            actions: Vec::new(),
            timeout: Timeout::Default,
            sound_name: Default::default(),
            id: None,
        }
    }

    #[cfg(target_os = "windows")]
    fn default() -> Notification {
        Notification {
            appname: exe_name(),
            summary: String::new(),
            subtitle: None,
            body: String::new(),
            icon: String::new(),
            actions: Vec::new(),
            timeout: Timeout::Default,
            sound_name: Default::default(),
            id: None,
            path_to_image: None,
            app_id: None,
        }
    }
}
