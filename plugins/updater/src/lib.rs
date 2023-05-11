use tauri::{
    plugin::{Builder as PluginBuilder, TauriPlugin},
    Manager, Runtime,
};

use tokio::sync::Mutex;

mod commands;
mod error;
mod updater;

pub use error::Error;
pub use updater::*;
pub type Result<T> = std::result::Result<T, Error>;

struct UpdaterState {
    target: Option<String>,
}

struct PendingUpdate<R: Runtime>(Mutex<Option<UpdateResponse<R>>>);

#[derive(Default)]
pub struct Builder {
    target: Option<String>,
}

/// Extension trait to use the updater on [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`].
pub trait UpdaterExt<R: Runtime> {
    /// Gets the updater builder to manually check if an update is available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_updater::UpdaterExt;
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle();
    ///     tauri::async_runtime::spawn(async move {
    ///         let response = handle.updater().check().await;
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    fn updater(&self) -> updater::UpdateBuilder<R>;
}

impl<R: Runtime, T: Manager<R>> UpdaterExt<R> for T {
    fn updater(&self) -> updater::UpdateBuilder<R> {
        updater::builder(self.app_handle())
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target.replace(target.into());
        self
    }

    pub fn build<R: Runtime>(self) -> TauriPlugin<R> {
        let target = self.target;
        PluginBuilder::<R>::new("updater")
            .setup(move |app, _api| {
                app.manage(UpdaterState { target });
                app.manage(PendingUpdate::<R>(Default::default()));
                Ok(())
            })
            .invoke_handler(tauri::generate_handler![
                commands::check,
                commands::download_and_install
            ])
            .build()
    }
}
