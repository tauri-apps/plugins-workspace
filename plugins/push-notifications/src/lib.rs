use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, RunEvent, Runtime,
};

pub use models::*;
use std::{collections::HashMap, sync::Mutex};

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::PushNotifications;
#[cfg(mobile)]
use mobile::PushNotifications;

/// Structure of push token state managed by the plugin.
#[derive(Default, Clone)]
pub struct PushTokenState {
    token: Option<Vec<u8>>,
    err: Option<String>,
}

/// Persistent push token store.
struct PushTokenStore(Mutex<HashMap<String, String>>);

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the push-notifications APIs.
pub trait PushNotificationsExt<R: Runtime> {
    fn push_notifications(&self) -> &PushNotifications<R>;
}

impl<R: Runtime, T: Manager<R>> PushNotificationsExt<R> for T {
    fn push_notifications(&self) -> &PushNotifications<R> {
        self.state::<PushNotifications<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("push-notifications")
        .invoke_handler(tauri::generate_handler![commands::push_token])
        .setup(|app, api| {
            // setup push token storage
            app.manage(PushTokenStore(Default::default()));
            app.manage(Mutex::new(PushTokenState::default()));
            #[cfg(mobile)]
            let push_notifications = mobile::init(app, api)?;
            #[cfg(desktop)]
            let push_notifications = desktop::init(app, api)?;
            app.manage(push_notifications);
            Ok(())
        })
        .on_event(|app, event| {
            match event {
                RunEvent::PushRegistration(token) => {
                    let state = app.state::<Mutex<PushTokenState>>();
                    let mut state = state.lock().unwrap();
                    let owned = token.to_owned();
                    state.token = Some(owned);
                }

                RunEvent::PushRegistrationFailed(err) => {
                    let state = app.state::<Mutex<PushTokenState>>();
                    let mut state = state.lock().unwrap();
                    let owned = err.to_owned();
                    state.err = Some(owned);
                }

                // @TODO: token persistence?
                _ => {}
            }
        })
        .build()
}
