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
use desktop::RevenueCat;
#[cfg(mobile)]
use mobile::RevenueCat;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the revenue-cat APIs.
pub trait RevenueCatExt<R: Runtime> {
  fn revenue_cat(&self) -> &RevenueCat<R>;
}

impl<R: Runtime, T: Manager<R>> crate::RevenueCatExt<R> for T {
  fn revenue_cat(&self) -> &RevenueCat<R> {
    self.state::<RevenueCat<R>>().inner()
  }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("revenue-cat")
    .invoke_handler(tauri::generate_handler![commands::ping])
    .setup(|app, api| {
      #[cfg(mobile)]
      let revenue_cat = mobile::init(app, api)?;
      #[cfg(desktop)]
      let revenue_cat = desktop::init(app, api)?;
      app.manage(revenue_cat);
      Ok(())
    })
    .build()
}
