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

/// What the notifications of this app share: the registered action types, who listens to
/// actions, and the shown notifications that can still be removed.
#[derive(Default)]
pub(crate) struct Shared {
    action_types: Mutex<HashMap<String, ActionType>>,
    handlers: Mutex<Vec<ActionHandler>>,
    listeners: Mutex<Vec<(String, Channel<Value>)>>,
    active: Mutex<HashMap<i32, imp::Shown>>,
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Shared {
    fn has_listeners(&self) -> bool {
        !lock(&self.handlers).is_empty()
            || lock(&self.listeners)
                .iter()
                .any(|(event, _)| event == ACTION_PERFORMED)
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
        for (event, channel) in lock(&self.listeners).iter() {
            if event == ACTION_PERFORMED {
                let _ = channel.send(payload.clone());
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
    /// to them, as far as the notification server reports it.
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
        for id in notifications {
            let shown = lock(&self.shared.active).remove(&id);
            if let Some(shown) = shown {
                shown.close();
            }
        }
        Ok(())
    }

    /// Removes all delivered notifications that can still be closed.
    pub fn remove_all_active(&self) -> crate::Result<()> {
        let shown = lock(&self.shared.active)
            .drain()
            .map(|(_, shown)| shown)
            .collect::<Vec<_>>();
        for shown in shown {
            shown.close();
        }
        Ok(())
    }

    /// Calls `handler` for every action the user performs on a notification of this app:
    /// a click on the notification itself (`tap`) or on one of its actions.
    ///
    /// Notifications shown before the first handler or listener existed report nothing.
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

    pub(crate) fn register_listener(&self, event: String, handler: Channel<Value>) {
        lock(&self.shared.listeners).push((event, handler));
    }

    pub(crate) fn remove_listener(&self, event: &str, channel_id: u32) {
        lock(&self.shared.listeners).retain(|(e, c)| !(e == event && c.id() == channel_id));
    }
}

mod imp {
    //! Types and functions related to desktop notifications.

    #[cfg(windows)]
    use std::path::MAIN_SEPARATOR as SEP;
    use std::sync::Arc;

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

    /// A shown notification that can be removed again.
    #[cfg(all(unix, not(target_os = "macos")))]
    pub(crate) struct Shown(notify_rust::NotificationHandle);

    /// A shown notification; it cannot be removed on this platform.
    #[cfg(any(windows, target_os = "macos"))]
    pub(crate) struct Shown;

    impl Shown {
        /// Removes the notification, where the platform allows it.
        pub(crate) fn close(self) {
            #[cfg(all(unix, not(target_os = "macos")))]
            self.0.close();
        }
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

        /// Shows the notification and waits on a thread of its own for what the user does with it;
        /// a click on the notification (`tap`) or on an action goes to `shared`'s listeners as an
        /// `actionPerformed` event carrying `data`. Closing it reports nothing.
        pub(crate) fn show_tracked(
            self,
            shared: Arc<Shared>,
            id: i32,
            data: Value,
        ) -> crate::Result<()> {
            #[allow(unused_mut)]
            let mut notification = self.build()?;
            // The Desktop Notifications Specification reports a click on the notification itself
            // as the action `default`, and only for a notification that offers that action.
            #[cfg(all(unix, not(target_os = "macos")))]
            notification.action("default", "");
            std::thread::Builder::new()
                .name("notification".into())
                .spawn(move || wait(notification, shared, id, data))?;
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

    /// Linux and the BSDs: the handle stays with `shared` until the notification is closed or
    /// clicked, so `remove_active` can close it meanwhile. The thread waits as long as the
    /// notification server keeps the notification.
    #[cfg(all(unix, not(target_os = "macos")))]
    fn wait(notification: notify_rust::Notification, shared: Arc<Shared>, id: i32, data: Value) {
        let Ok(handle) = notification.show() else {
            return;
        };
        let server_id = handle.id();
        lock(&shared.active).insert(id, Shown(handle));
        let _ = notify_rust::handle_action(server_id, |response| {
            // A later notification with the same id may have taken the entry meanwhile.
            let mut active = lock(&shared.active);
            if active
                .get(&id)
                .is_some_and(|shown| shown.0.id() == server_id)
            {
                active.remove(&id);
            }
            drop(active);
            let action = match response {
                notify_rust::ActionResponse::Custom("default") => "tap",
                notify_rust::ActionResponse::Custom(action) => action,
                notify_rust::ActionResponse::Closed(_) => return,
            };
            shared.dispatch(performed(action, &data));
        });
    }

    /// macOS and Windows: `notify-rust` sends the notification and blocks until the user acts on
    /// it or it goes away.
    #[cfg(any(windows, target_os = "macos"))]
    fn wait(notification: notify_rust::Notification, shared: Arc<Shared>, _id: i32, data: Value) {
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
