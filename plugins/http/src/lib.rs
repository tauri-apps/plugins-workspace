pub use reqwest as client;
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

use std::{collections::HashMap, sync::Mutex};

mod commands;
mod error;
mod scope;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;
type ClientId = u32;

pub struct Http<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    pub(crate) clients: Mutex<HashMap<ClientId, commands::Client>>,
    pub(crate) scope: scope::Scope,
}

impl<R: Runtime> Http<R> {}

pub trait HttpExt<R: Runtime> {
    fn http(&self) -> &Http<R>;
}

impl<R: Runtime, T: Manager<R>> HttpExt<R> for T {
    fn http(&self) -> &Http<R> {
        self.state::<Http<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("http")
        .invoke_handler(tauri::generate_handler![
            commands::create_client,
            commands::drop_client,
            commands::request
        ])
        .setup(|app, _api| {
            app.manage(Http {
                app: app.clone(),
                clients: Default::default(),
                scope: scope::Scope::new(&app.config().tauri.allowlist.http.scope),
            });
            Ok(())
        })
        .build()
}
