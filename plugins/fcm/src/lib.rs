use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod config;
mod commands;
mod error;
mod models;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Fcm;
#[cfg(mobile)]
use mobile::Fcm;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the fcm APIs.
pub trait FcmExt<R: Runtime> {
    fn fcm(&self) -> &Fcm<R>;
}

impl<R: Runtime, T: Manager<R>> crate::FcmExt<R> for T {
    fn fcm(&self) -> &Fcm<R> {
        self.state::<Fcm<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R, config::Config> {
  Builder::<R, config::Config>::new("fcm")
    .invoke_handler(tauri::generate_handler![
      commands::get_latest_notification_data,
      commands::get_token,
      commands::subscribe_to_topic
    ])
    .setup(|app, api| {
      #[cfg(mobile)]
      let fcm = mobile::init(app, api)?;
      #[cfg(desktop)]
      let fcm = desktop::init(app, api)?;
      app.manage(fcm);
      Ok(())
    })
    .build()
}
