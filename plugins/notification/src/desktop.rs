// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    sync::{Arc, Mutex, MutexGuard, PoisonError},
};

use serde::de::DeserializeOwned;
use serde_json::Value;
use tauri::{
    AppHandle, Manager, Runtime,
    ipc::Channel,
    plugin::{PermissionState, PluginApi},
};

use crate::{ActionPerformed, ActionType, NotificationBuilder};

/// The event of the JavaScript `onAction` listeners.
const ACTION_PERFORMED: &str = "actionPerformed";

/// Initializes the desktop implementation of the notification APIs.
pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Notification<R>> {
    Ok(Notification {
        app: app.clone(),
        shared: Arc::default(),
    })
}

type ActionHandler = Arc<dyn Fn(&ActionPerformed) + Send + Sync>;

/// A JavaScript listener, with the webview and the window it was registered from.
struct Listener {
    event: String,
    channel: Channel<Value>,
    webview: String,
    window: String,
}

/// What the notifications of this app share: the registered action types, who listens to
/// actions, and the shown notifications whose actions are reported.
#[derive(Default)]
pub(crate) struct Shared {
    action_types: Mutex<HashMap<String, ActionType>>,
    handlers: Mutex<Vec<ActionHandler>>,
    listeners: Mutex<Vec<Listener>>,
    #[cfg(all(unix, not(target_os = "macos")))]
    tracker: imp::Tracker,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Shared {
    fn has_listeners(&self) -> bool {
        !lock(&self.handlers).is_empty()
            || lock(&self.listeners)
                .iter()
                .any(|listener| listener.event == ACTION_PERFORMED)
    }

    fn retain_listeners(&self, keep: impl Fn(&Listener) -> bool) {
        lock(&self.listeners).retain(keep);
    }

    /// Hands an action to the Rust handlers and the JavaScript listeners. The handlers run
    /// without a lock held, so they may register handlers or remove notifications themselves.
    fn dispatch(&self, payload: Value) {
        if let Ok(performed) = serde_json::from_value::<ActionPerformed>(payload.clone()) {
            let handlers = lock(&self.handlers).clone();
            for handler in handlers {
                handler(&performed);
            }
        }
        for listener in lock(&self.listeners).iter() {
            if listener.event == ACTION_PERFORMED {
                let _ = listener.channel.send(payload.clone());
            }
        }
    }
}

/// Access to the notification APIs.
///
/// You can get an instance of this type via [`NotificationExt`](crate::NotificationExt)
pub struct Notification<R: Runtime> {
    app: AppHandle<R>,
    shared: Arc<Shared>,
}

impl<R: Runtime> crate::NotificationBuilder<R> {
    /// Shows the notification.
    ///
    /// When no title was set with [`Self::title`], the `productName` from the Tauri configuration is used instead.
    /// Only the title, body, icon, sound and the actions of the [action type](Self::action_type_id)
    /// of the notification are used on desktop; the scheduling and grouping options are ignored.
    ///
    /// When a handler is registered with [`Notification::on_action`] or a JavaScript `onAction`
    /// listener exists, a click on the notification (`tap`) or on one of its actions is reported
    /// to them, as far as the notification server reports it. Otherwise the actions of the
    /// notification are not shown, since nothing would receive them.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows**: a click is only reported while the toast is on screen, not once it moved to
    ///   the Action Center.
    /// - **macOS**: two or more actions are shown in an "Options" menu, and a click on the menu
    ///   button itself reports the first action. Actions are told apart by their title.
    ///
    /// The notification is dispatched on a background task, so this returns as soon as the payload is prepared.
    ///
    /// # Errors
    ///
    /// Returns an error when the notification could not be prepared,
    /// e.g. when the path of the running executable cannot be resolved on Windows.
    pub fn show(self) -> crate::Result<()> {
        let shared = Arc::clone(&self.app.state::<Notification<R>>().shared);
        let mut notification = imp::Notification::new(self.app.config().identifier.clone());

        if let Some(title) = self
            .data
            .title
            .clone()
            .or_else(|| self.app.config().product_name.clone())
        {
            notification = notification.title(title);
        }
        if let Some(body) = self.data.body.clone() {
            notification = notification.body(body);
        }
        if let Some(icon) = self.data.icon.clone() {
            notification = notification.icon(icon);
        }
        if let Some(sound) = self.data.sound.clone() {
            notification = notification.sound(sound);
        }
        if shared.has_listeners() {
            if let Some(action_type) = self
                .data
                .action_type_id
                .as_ref()
                .and_then(|id| lock(&shared.action_types).get(id).cloned())
            {
                for action in action_type.actions() {
                    notification = notification.action(action.id(), action.title());
                }
            }
            let id = self.data.id;
            let data = serde_json::to_value(&self.data).unwrap_or(Value::Null);
            notification.show_tracked(shared, id, data)?;
        } else {
            notification.show()?;
        }

        Ok(())
    }
}

impl<R: Runtime> Notification<R> {
    /// Creates a new builder for a notification.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_notification::NotificationExt;
    ///
    /// fn notify<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
    ///   app.notification()
    ///     .builder()
    ///     .title("Tauri")
    ///     .body("Tauri is awesome!")
    ///     .show()
    ///     .unwrap();
    /// }
    /// ```
    pub fn builder(&self) -> NotificationBuilder<R> {
        NotificationBuilder::new(self.app.clone())
    }

    /// Requests the permission to send notifications.
    ///
    /// Desktop applications do not need to ask for this permission,
    /// so this always resolves to [`PermissionState::Granted`] without prompting the user.
    pub fn request_permission(&self) -> crate::Result<PermissionState> {
        Ok(PermissionState::Granted)
    }

    /// Checks whether the permission to send notifications was granted.
    ///
    /// Desktop applications do not need to ask for this permission,
    /// so this always resolves to [`PermissionState::Granted`].
    pub fn permission_state(&self) -> crate::Result<PermissionState> {
        Ok(PermissionState::Granted)
    }

    /// Registers the action types a notification can reference
    /// through [`NotificationBuilder::action_type_id`](crate::NotificationBuilder::action_type_id).
    ///
    /// On desktop the actions of a type become the buttons of the notification; only their
    /// identifier and title are used. A type registered again replaces the previous one.
    pub fn register_action_types(&self, types: Vec<ActionType>) -> crate::Result<()> {
        let mut registered = lock(&self.shared.action_types);
        for action_type in types {
            registered.insert(action_type.id().to_owned(), action_type);
        }
        Ok(())
    }

    /// Removes the delivered notifications with the given identifiers.
    ///
    /// ## Platform-specific
    ///
    /// - **Linux and the BSDs**: closes notifications shown while an action listener existed.
    /// - **macOS and Windows**: does nothing, `notify-rust` cannot remove a shown notification there.
    pub fn remove_active(&self, notifications: Vec<i32>) -> crate::Result<()> {
        imp::close(&self.shared, Some(&notifications));
        Ok(())
    }

    /// Removes all delivered notifications that can still be closed.
    pub fn remove_all_active(&self) -> crate::Result<()> {
        imp::close(&self.shared, None);
        Ok(())
    }

    /// Calls `handler` for every action the user performs on a notification of this app:
    /// a click on the notification itself (`tap`) or on one of its actions.
    ///
    /// Notifications shown before the first handler or listener existed report nothing.
    /// The handler runs on a background thread and should return quickly.
    ///
    /// # Errors
    ///
    /// Never on desktop; the signature matches the mobile one.
    pub fn on_action<F: Fn(&ActionPerformed) + Send + Sync + 'static>(
        &self,
        handler: F,
    ) -> crate::Result<()> {
        lock(&self.shared.handlers).push(Arc::new(handler));
        Ok(())
    }

    pub(crate) fn register_listener(
        &self,
        event: String,
        channel: Channel<Value>,
        webview: &tauri::Webview<R>,
    ) {
        lock(&self.shared.listeners).push(Listener {
            event,
            channel,
            webview: webview.label().to_owned(),
            window: webview.window().label().to_owned(),
        });
    }

    pub(crate) fn remove_listener(&self, event: &str, channel_id: u32) {
        lock(&self.shared.listeners)
            .retain(|listener| !(listener.event == event && listener.channel.id() == channel_id));
    }

    /// Drops the listeners of a webview that navigates away, since its page no longer exists.
    pub(crate) fn remove_webview_listeners(&self, webview: &str) {
        self.shared
            .retain_listeners(|listener| listener.webview != webview);
    }

    /// Drops the listeners of the webviews of a destroyed window.
    pub(crate) fn remove_window_listeners(&self, window: &str) {
        self.shared
            .retain_listeners(|listener| listener.window != window);
    }
}

mod imp {
    //! Types and functions related to desktop notifications.

    #[cfg(windows)]
    use std::path::MAIN_SEPARATOR as SEP;
    use std::sync::Arc;
    #[cfg(all(unix, not(target_os = "macos")))]
    use std::{
        collections::HashMap,
        sync::{Mutex, OnceLock, Weak},
    };

    use serde_json::{Value, json};

    use super::Shared;
    #[cfg(all(unix, not(target_os = "macos")))]
    use super::lock;

    /// The desktop notification definition.
    ///
    /// Allows you to construct a Notification data and send it.
    ///
    /// # Examples
    /// ```rust,no_run
    /// use tauri_plugin_notification::NotificationExt;
    /// // first we build the application to access the Tauri configuration
    /// let app = tauri::Builder::default()
    ///   // on an actual app, remove the string argument
    ///   .build(tauri::generate_context!("test/tauri.conf.json"))
    ///   .expect("error while building tauri application");
    ///
    /// // shows a notification with the given title and body
    /// app.notification()
    ///   .builder()
    ///   .title("New message")
    ///   .body("You've got a new message.")
    ///   .show();
    ///
    /// // run the app
    /// app.run(|_app_handle, _event| {});
    /// ```
    #[allow(dead_code)]
    #[derive(Debug, Default)]
    pub struct Notification {
        /// The notification body.
        body: Option<String>,
        /// The notification title.
        title: Option<String>,
        /// The notification icon.
        icon: Option<String>,
        /// The notification sound.
        sound: Option<String>,
        /// The notification identifier
        identifier: String,
        /// The actions, as identifier and title.
        actions: Vec<(String, String)>,
    }

    /// Linux and the BSDs: the notifications whose actions are reported. A single session bus
    /// connection, opened with the first of them, receives the signals of all of them on a thread
    /// of its own, so a tracked notification costs no thread or connection of its own.
    #[cfg(all(unix, not(target_os = "macos")))]
    #[derive(Default)]
    pub(crate) struct Tracker {
        /// The connection the signals arrive on; `None` when it could not be set up.
        connection: OnceLock<Option<zbus::blocking::Connection>>,
        /// The tracked notifications the server still shows, by server id: their identifier and
        /// the payload their actions carry.
        pub(super) shown: Mutex<HashMap<u32, (i32, Value)>>,
    }

    /// The payload of an `actionPerformed` event, as the mobile plugins send it.
    pub(super) fn performed(action_id: &str, notification: &Value) -> Value {
        json!({
            "actionId": action_id,
            "inputValue": null,
            "notification": notification,
        })
    }

    impl Notification {
        /// Initializes a instance of a Notification.
        pub fn new(identifier: impl Into<String>) -> Self {
            Self {
                identifier: identifier.into(),
                ..Default::default()
            }
        }

        /// Sets the notification body.
        #[must_use]
        pub fn body(mut self, body: impl Into<String>) -> Self {
            self.body = Some(body.into());
            self
        }

        /// Sets the notification title.
        #[must_use]
        pub fn title(mut self, title: impl Into<String>) -> Self {
            self.title = Some(title.into());
            self
        }

        /// Sets the notification icon.
        #[must_use]
        pub fn icon(mut self, icon: impl Into<String>) -> Self {
            self.icon = Some(icon.into());
            self
        }

        /// Sets the notification sound file.
        #[must_use]
        pub fn sound(mut self, sound: impl Into<String>) -> Self {
            self.sound = Some(sound.into());
            self
        }

        /// Adds an action button.
        #[must_use]
        pub fn action(mut self, id: impl Into<String>, title: impl Into<String>) -> Self {
            self.actions.push((id.into(), title.into()));
            self
        }

        /// Shows the notification.
        ///
        /// # Examples
        ///
        /// ```no_run
        /// use tauri_plugin_notification::NotificationExt;
        ///
        /// tauri::Builder::default()
        ///   .setup(|app| {
        ///     app.notification()
        ///       .builder()
        ///       .title("Tauri")
        ///       .body("Tauri is awesome!")
        ///       .show()
        ///       .unwrap();
        ///     Ok(())
        ///   })
        ///   .run(tauri::generate_context!("test/tauri.conf.json"))
        ///   .expect("error while running tauri application");
        /// ```
        pub fn show(self) -> crate::Result<()> {
            let notification = self.build()?;
            tauri::async_runtime::spawn(async move {
                let _ = notification.show();
            });

            Ok(())
        }

        /// Shows the notification and reports what the user does with it: a click on the
        /// notification (`tap`) or on an action goes to `shared`'s listeners as an
        /// `actionPerformed` event carrying `data`. Closing it reports nothing.
        #[cfg(all(unix, not(target_os = "macos")))]
        pub(crate) fn show_tracked(
            self,
            shared: Arc<Shared>,
            id: i32,
            data: Value,
        ) -> crate::Result<()> {
            let mut notification = self.build()?;
            // The Desktop Notifications Specification reports a click on the notification itself
            // as the action `default`, and only for a notification that offers that action.
            notification.action("default", "");
            tauri::async_runtime::spawn_blocking(move || track(notification, &shared, id, data));
            Ok(())
        }

        /// Shows the notification and waits on a thread of its own for what the user does with it;
        /// a click on the notification (`tap`) or on an action goes to `shared`'s listeners as an
        /// `actionPerformed` event carrying `data`. Closing it reports nothing.
        #[cfg(any(windows, target_os = "macos"))]
        pub(crate) fn show_tracked(
            self,
            shared: Arc<Shared>,
            _id: i32,
            data: Value,
        ) -> crate::Result<()> {
            let notification = self.build()?;
            std::thread::Builder::new()
                .name("notification".into())
                .spawn(move || wait(notification, shared, data))?;
            Ok(())
        }

        fn build(self) -> crate::Result<notify_rust::Notification> {
            let mut notification = notify_rust::Notification::new();
            for (id, title) in &self.actions {
                notification.action(id, title);
            }
            if let Some(body) = self.body {
                notification.body(&body);
            }
            if let Some(title) = self.title {
                notification.summary(&title);
            }
            if let Some(icon) = self.icon {
                notification.icon(&icon);
            } else {
                notification.auto_icon();
            }
            if let Some(sound) = self.sound {
                notification.sound_name(&sound);
            }
            #[cfg(windows)]
            {
                let exe = tauri::utils::platform::current_exe()?;
                let exe_dir = exe.parent().expect("failed to get exe directory");
                let curr_dir = exe_dir.display().to_string();
                // set the notification's System.AppUserModel.ID only when running the installed app
                if !(curr_dir.ends_with(format!("{SEP}target{SEP}debug").as_str())
                    || curr_dir.ends_with(format!("{SEP}target{SEP}release").as_str()))
                {
                    notification.app_id(&self.identifier);
                }
            }
            #[cfg(target_os = "macos")]
            {
                let _ = notify_rust::set_application(if tauri::is_dev() {
                    "com.apple.Terminal"
                } else {
                    &self.identifier
                });
            }

            Ok(notification)
        }

        /// Shows the notification. Same as [`Self::show`].
        #[cfg(feature = "windows7-compat")]
        #[allow(dead_code)]
        #[cfg_attr(docsrs, doc(cfg(feature = "windows7-compat")))]
        #[deprecated = "Tauri no longer supports Windows 7, use `Self::show` instead."]
        pub fn notify<R: tauri::Runtime>(self, _app: &tauri::AppHandle<R>) -> crate::Result<()> {
            self.show()
        }
    }

    /// The bus name, object path and interface of the Desktop Notifications Specification.
    #[cfg(all(unix, not(target_os = "macos")))]
    pub(super) const BUS: (&str, &str, &str) = (
        "org.freedesktop.Notifications",
        "/org/freedesktop/Notifications",
        "org.freedesktop.Notifications",
    );

    /// Linux and the BSDs: shows the notification and records it, so the listener thread knows
    /// which payload its signals carry.
    #[cfg(all(unix, not(target_os = "macos")))]
    fn track(notification: notify_rust::Notification, shared: &Arc<Shared>, id: i32, data: Value) {
        let tracker = &shared.tracker;
        let Some(connection) = tracker
            .connection
            .get_or_init(|| listen(Arc::downgrade(shared)))
        else {
            let _ = notification.show();
            return;
        };
        // Held while the notification is sent, so a signal that arrives before the server id is
        // recorded waits for it instead of being missed.
        let mut shown = lock(&tracker.shown);
        if let Ok(server_id) = notify(connection, &notification) {
            shown.insert(server_id, (id, data));
        }
    }

    /// Linux and the BSDs: sends the notification on the connection the signals arrive on, since
    /// some servers, like dunst, send the signals of a notification only to the connection that
    /// sent it. Returns the server id of the notification.
    #[cfg(all(unix, not(target_os = "macos")))]
    fn notify(
        connection: &zbus::blocking::Connection,
        notification: &notify_rust::Notification,
    ) -> zbus::Result<u32> {
        // the only hint the plugin sets
        let hints = notification
            .hints
            .iter()
            .filter_map(|hint| match hint {
                notify_rust::Hint::SoundName(name) => {
                    Some(("sound-name", zbus::zvariant::Value::from(name.as_str())))
                }
                _ => None,
            })
            .collect::<HashMap<_, _>>();
        let (destination, path, interface) = BUS;
        connection
            .call_method(
                Some(destination),
                path,
                Some(interface),
                "Notify",
                &(
                    &notification.appname,
                    0u32,
                    &notification.icon,
                    &notification.summary,
                    &notification.body,
                    &notification.actions,
                    hints,
                    -1i32,
                ),
            )?
            .body()
            .deserialize()
    }

    /// Linux and the BSDs: connects to the session bus and subscribes to the signals of the
    /// notification server, then handles them on a thread of its own until the plugin goes away.
    /// The subscription is in place when this returns, so no signal of a later notification is
    /// missed.
    #[cfg(all(unix, not(target_os = "macos")))]
    fn listen(shared: Weak<Shared>) -> Option<zbus::blocking::Connection> {
        let (_, path, interface) = BUS;
        let connection = zbus::blocking::Connection::session().ok()?;
        let rule = zbus::MatchRule::builder()
            .msg_type(zbus::message::Type::Signal)
            .path(path)
            .ok()?
            .interface(interface)
            .ok()?
            .build();
        let signals =
            zbus::blocking::MessageIterator::for_match_rule(rule, &connection, None).ok()?;
        std::thread::Builder::new()
            .name("notification-actions".into())
            .spawn(move || {
                for signal in signals.flatten() {
                    let Some(shared) = shared.upgrade() else {
                        return;
                    };
                    on_signal(&shared, &signal);
                }
            })
            .ok()?;
        Some(connection)
    }

    /// Linux and the BSDs: reports an `ActionInvoked` signal of a tracked notification and forgets
    /// the notification on `NotificationClosed`. Signals of other notifications are ignored.
    #[cfg(all(unix, not(target_os = "macos")))]
    pub(super) fn on_signal(shared: &Shared, signal: &zbus::message::Message) {
        let header = signal.header();
        let body = signal.body();
        let (server_id, action) = match header.member().map(|member| member.as_str()) {
            Some("ActionInvoked") => match body.deserialize::<(u32, String)>() {
                Ok((server_id, action)) => (server_id, action),
                Err(_) => return,
            },
            Some("NotificationClosed") => {
                if let Ok((server_id, _reason)) = body.deserialize::<(u32, u32)>() {
                    lock(&shared.tracker.shown).remove(&server_id);
                }
                return;
            }
            _ => return,
        };
        // The entry stays until the server closes the notification, which it may keep showing.
        let data = lock(&shared.tracker.shown)
            .get(&server_id)
            .map(|(_, data)| data.clone());
        if let Some(data) = data {
            let action = if action == "default" { "tap" } else { &action };
            shared.dispatch(performed(action, &data));
        }
    }

    /// Linux and the BSDs: closes the tracked notifications with the given identifiers, or all of
    /// them. They are forgotten when the server reports them closed.
    #[cfg(all(unix, not(target_os = "macos")))]
    pub(super) fn close(shared: &Shared, ids: Option<&[i32]>) {
        let Some(Some(connection)) = shared.tracker.connection.get() else {
            return;
        };
        let server_ids = lock(&shared.tracker.shown)
            .iter()
            .filter(|(_, (id, _))| ids.is_none_or(|ids| ids.contains(id)))
            .map(|(server_id, _)| *server_id)
            .collect::<Vec<_>>();
        let (destination, path, interface) = BUS;
        for server_id in server_ids {
            let _ = connection.call_method(
                Some(destination),
                path,
                Some(interface),
                "CloseNotification",
                &(server_id,),
            );
        }
    }

    /// macOS and Windows: `notify-rust` cannot remove a shown notification.
    #[cfg(any(windows, target_os = "macos"))]
    pub(super) fn close(_shared: &Shared, _ids: Option<&[i32]>) {}

    /// macOS and Windows: `notify-rust` sends the notification and blocks until the user acts on
    /// it or it goes away.
    #[cfg(any(windows, target_os = "macos"))]
    fn wait(notification: notify_rust::Notification, shared: Arc<Shared>, data: Value) {
        let Ok(handle) = notification.show() else {
            return;
        };
        let _ = handle.wait_for_response(|response: &notify_rust::NotificationResponse| {
            let action = match response {
                notify_rust::NotificationResponse::Default => "tap",
                notify_rust::NotificationResponse::Action(action) => action.as_str(),
                _ => return,
            };
            shared.dispatch(performed(action, &data));
        });
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use super::*;
    use crate::NotificationData;

    #[test]
    fn a_performed_action_reaches_the_rust_handlers_with_its_notification() {
        let shared = Shared::default();
        assert!(!shared.has_listeners());
        let seen = Arc::new(Mutex::new(Vec::new()));
        let record = Arc::clone(&seen);
        lock(&shared.handlers).push(Arc::new(move |performed: &ActionPerformed| {
            record.lock().unwrap().push((
                performed.action_id().to_owned(),
                performed.notification().map(|n| n.id()),
                performed
                    .notification()
                    .and_then(|n| n.title().map(str::to_owned)),
            ));
        }));
        assert!(shared.has_listeners());

        let data = NotificationData {
            id: 7,
            title: Some("Standup".into()),
            ..Default::default()
        };
        let data = serde_json::to_value(&data).unwrap();
        shared.dispatch(imp::performed("tap", &data));
        shared.dispatch(imp::performed("snooze", &data));
        assert_eq!(
            *seen.lock().unwrap(),
            [
                ("tap".to_owned(), Some(7), Some("Standup".to_owned())),
                ("snooze".to_owned(), Some(7), Some("Standup".to_owned())),
            ]
        );
    }

    #[test]
    fn an_unreadable_notification_still_reports_the_action() {
        let performed: ActionPerformed = serde_json::from_value(serde_json::json!({
            "actionId": "tap",
            "inputValue": null,
            "notification": { "id": "not a number" },
        }))
        .unwrap();
        assert_eq!(performed.action_id(), "tap");
        assert!(performed.notification().is_none());
    }

    fn listener(webview: &str, window: &str) -> Listener {
        Listener {
            event: ACTION_PERFORMED.into(),
            channel: Channel::new(|_| Ok(())),
            webview: webview.into(),
            window: window.into(),
        }
    }

    #[test]
    fn the_listeners_of_a_gone_page_stop_tracking() {
        let shared = Shared::default();
        lock(&shared.listeners).extend([listener("main", "main"), listener("side", "other")]);
        shared.retain_listeners(|listener| listener.webview != "main");
        assert!(shared.has_listeners());
        shared.retain_listeners(|listener| listener.window != "other");
        assert!(!shared.has_listeners());
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    fn signal(
        member: &str,
        body: &(impl serde::Serialize + zbus::zvariant::DynamicType),
    ) -> zbus::message::Message {
        let (_, path, interface) = imp::BUS;
        zbus::message::Message::signal(path, interface, member)
            .unwrap()
            .build(body)
            .unwrap()
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    #[test]
    fn the_signals_of_a_tracked_notification_are_reported_until_it_closes() {
        let shared = Shared::default();
        let seen = Arc::new(Mutex::new(Vec::new()));
        let record = Arc::clone(&seen);
        lock(&shared.handlers).push(Arc::new(move |performed: &ActionPerformed| {
            record
                .lock()
                .unwrap()
                .push(performed.action_id().to_owned());
        }));
        lock(&shared.tracker.shown).insert(41, (7, serde_json::json!({ "id": 7 })));

        imp::on_signal(&shared, &signal("ActionInvoked", &(41u32, "default")));
        imp::on_signal(&shared, &signal("ActionInvoked", &(41u32, "snooze")));
        // another notification, maybe of another app
        imp::on_signal(&shared, &signal("ActionInvoked", &(42u32, "default")));
        imp::on_signal(&shared, &signal("NotificationClosed", &(41u32, 2u32)));
        imp::on_signal(&shared, &signal("ActionInvoked", &(41u32, "default")));

        assert_eq!(*seen.lock().unwrap(), ["tap", "snooze"]);
        assert!(lock(&shared.tracker.shown).is_empty());
    }

    #[test]
    fn a_handler_may_register_another_handler() {
        let shared = Arc::new(Shared::default());
        let inner = Arc::clone(&shared);
        lock(&shared.handlers).push(Arc::new(move |_: &ActionPerformed| {
            lock(&inner.handlers).push(Arc::new(|_: &ActionPerformed| {}));
        }));
        shared.dispatch(imp::performed("tap", &Value::Null));
        assert_eq!(lock(&shared.handlers).len(), 2);
    }
}
