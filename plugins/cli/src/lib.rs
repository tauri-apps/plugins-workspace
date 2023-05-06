use tauri::{
    plugin::{Builder, PluginApi, TauriPlugin},
    AppHandle, Manager, Runtime, State,
};

mod config;
mod error;
mod parser;

use config::{Arg, Config};
pub use error::Error;
type Result<T> = std::result::Result<T, Error>;

// TODO: use PluginApi#app when 2.0.0-alpha.9 is released
pub struct Cli<R: Runtime>(PluginApi<R, Config>, AppHandle<R>);

impl<R: Runtime> Cli<R> {
    pub fn matches(&self) -> Result<parser::Matches> {
        parser::get_matches(self.0.config(), self.1.package_info())
    }
}

pub trait CliExt<R: Runtime> {
    fn cli(&self) -> &Cli<R>;
}

impl<R: Runtime, T: Manager<R>> CliExt<R> for T {
    fn cli(&self) -> &Cli<R> {
        self.state::<Cli<R>>().inner()
    }
}

#[tauri::command]
fn cli_matches<R: Runtime>(_app: AppHandle<R>, cli: State<'_, Cli<R>>) -> Result<parser::Matches> {
    cli.matches()
}

pub fn init<R: Runtime>() -> TauriPlugin<R, Config> {
    Builder::new("cli")
        .invoke_handler(tauri::generate_handler![cli_matches])
        .setup(|app, api| {
            app.manage(Cli(api, app.clone()));
            Ok(())
        })
        .build()
}
