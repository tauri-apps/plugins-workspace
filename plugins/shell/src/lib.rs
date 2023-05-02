use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use process::{Command, CommandChild};
use regex::Regex;
use scope::{Scope, ScopeAllowedCommand, ScopeConfig};
use tauri::{
    plugin::{Builder, TauriPlugin},
    utils::config::{ShellAllowedArg, ShellAllowedArgs, ShellAllowlistOpen, ShellAllowlistScope},
    AppHandle, Manager, RunEvent, Runtime,
};

mod commands;
mod error;
mod open;
pub mod process;
mod scope;

pub use error::Error;
type Result<T> = std::result::Result<T, Error>;
type ChildStore = Arc<Mutex<HashMap<u32, CommandChild>>>;

pub struct Shell<R: Runtime> {
    #[allow(dead_code)]
    app: AppHandle<R>,
    scope: Scope,
    children: ChildStore,
}

impl<R: Runtime> Shell<R> {
    /// Creates a new Command for launching the given program.
    pub fn command(&self, program: impl Into<String>) -> Command {
        Command::new(program)
    }

    /// Creates a new Command for launching the given sidecar program.
    ///
    /// A sidecar program is a embedded external binary in order to make your application work
    /// or to prevent users having to install additional dependencies (e.g. Node.js, Python, etc).
    pub fn sidecar(&self, program: impl Into<String>) -> Result<Command> {
        Command::new_sidecar(program)
    }

    /// Open a (url) path with a default or specific browser opening program.
    ///
    /// See [`crate::api::shell::open`] for how it handles security-related measures.
    pub fn open(&self, path: String, with: Option<open::Program>) -> Result<()> {
        open::open(&self.scope, path, with).map_err(Into::into)
    }
}

pub trait ShellExt<R: Runtime> {
    fn shell(&self) -> &Shell<R>;
}

impl<R: Runtime, T: Manager<R>> ShellExt<R> for T {
    fn shell(&self) -> &Shell<R> {
        self.state::<Shell<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("shell")
        .invoke_handler(tauri::generate_handler![
            commands::execute,
            commands::stdin_write,
            commands::kill,
            commands::open
        ])
        .setup(|app, _api| {
            app.manage(Shell {
                app: app.clone(),
                children: Default::default(),
                scope: Scope::new(
                    app,
                    shell_scope(
                        app.config().tauri.allowlist.shell.scope.clone(),
                        &app.config().tauri.allowlist.shell.open,
                    ),
                ),
            });
            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::Exit = event {
                let shell = app.state::<Shell<R>>();
                let children = {
                    let mut lock = shell.children.lock().unwrap();
                    std::mem::take(&mut *lock)
                };
                for child in children.into_values() {
                    let _ = child.kill();
                }
            }
        })
        .build()
}

fn shell_scope(scope: ShellAllowlistScope, open: &ShellAllowlistOpen) -> ScopeConfig {
    let shell_scopes = get_allowed_clis(scope);

    let shell_scope_open = match open {
        ShellAllowlistOpen::Flag(false) => None,
        ShellAllowlistOpen::Flag(true) => {
            Some(Regex::new(r#"^((mailto:\w+)|(tel:\w+)|(https?://\w+)).+"#).unwrap())
        }
        ShellAllowlistOpen::Validate(validator) => {
            let validator =
                Regex::new(validator).unwrap_or_else(|e| panic!("invalid regex {validator}: {e}"));
            Some(validator)
        }
        _ => panic!("unknown shell open format, unable to prepare"),
    };

    ScopeConfig {
        open: shell_scope_open,
        scopes: shell_scopes,
    }
}

fn get_allowed_clis(scope: ShellAllowlistScope) -> HashMap<String, ScopeAllowedCommand> {
    scope
        .0
        .into_iter()
        .map(|scope| {
            let args = match scope.args {
                ShellAllowedArgs::Flag(true) => None,
                ShellAllowedArgs::Flag(false) => Some(Vec::new()),
                ShellAllowedArgs::List(list) => {
                    let list = list.into_iter().map(|arg| match arg {
                        ShellAllowedArg::Fixed(fixed) => scope::ScopeAllowedArg::Fixed(fixed),
                        ShellAllowedArg::Var { validator } => {
                            let validator = Regex::new(&validator)
                                .unwrap_or_else(|e| panic!("invalid regex {validator}: {e}"));
                            scope::ScopeAllowedArg::Var { validator }
                        }
                        _ => panic!("unknown shell scope arg, unable to prepare"),
                    });
                    Some(list.collect())
                }
                _ => panic!("unknown shell scope command, unable to prepare"),
            };

            (
                scope.name,
                ScopeAllowedCommand {
                    command: scope.command,
                    args,
                    sidecar: scope.sidecar,
                },
            )
        })
        .collect()
}
