// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;

use crate::{Builder, SingleInstanceCallback};
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Config, Manager, RunEvent, Runtime,
};
use zbus::{blocking::Connection, interface};

struct ConnectionHandle(Connection);

struct SingleInstanceDBus<R: Runtime> {
    callback: Box<SingleInstanceCallback<R>>,
    app_handle: AppHandle<R>,
}

#[interface(name = "org.SingleInstance.DBus")]
impl<R: Runtime> SingleInstanceDBus<R> {
    fn execute_callback(&mut self, argv: Vec<String>, cwd: String) {
        (self.callback)(&self.app_handle, argv, cwd);
    }
}

#[cfg(feature = "semver")]
fn dbus_id(config: &Config, version: semver::Version) -> String {
    let mut id = config.identifier.replace(['.', '-'], "_");
    id.push('_');
    id.push_str(semver_compat_string(version).as_str());
    id
}

#[cfg(not(feature = "semver"))]
fn dbus_id(config: &Config) -> String {
    config.identifier.clone()
}

fn dbus_path(config: &Config) -> String {
    config.identifier.replace(['.', '-'], "_")
}
pub fn init<R: Runtime>(f: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    Builder::new().build(f)
}

pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M, custom_dbus_id: Option<String>) {
    if let Some(connection) = manager.try_state::<ConnectionHandle>() {
        #[cfg(feature = "semver")]
        let id = dbus_id(
            manager.config(),
            manager.app_handle().package_info().version.clone(),
        );
        #[cfg(not(feature = "semver"))]
        let id = dbus_id(manager.config());

        let dbus_name = if let Some(id) = custom_dbus_id {
            id.clone()
        } else {
            format!("{id}.SingleInstance",)
        };
        let _ = connection.0.release_name(dbus_name);
    }
}

impl Builder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom DBus ID
    #[allow(dead_code)]
    pub fn dbus_id(mut self, dbus_id: String) -> Self {
        self.dbus_id = Some(dbus_id);
        self
    }
    pub fn build<R: Runtime>(self, f: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
        let custom_dbus_id = self.dbus_id.clone();
        plugin::Builder::new("single-instance")
            .setup(|app, _api| {
                let custom_id = self.dbus_id;
                #[cfg(feature = "semver")]
                let mut id = dbus_id(app.config(), app.package_info().version.clone());
                #[cfg(not(feature = "semver"))]
                let mut id = dbus_id(app.config());

                if let Some(dbus_id) = custom_id.clone() {
                    id = dbus_id;
                }

                let single_instance_dbus = SingleInstanceDBus {
                    callback: f,
                    app_handle: app.clone(),
                };

                app.manage(Builder {
                    dbus_id: custom_id.clone(),
                });

                let path = dbus_path(app.config());
                let dbus_path = format!("/{path}/SingleInstance");
                let dbus_name = if custom_id.is_some() {
                    id
                } else {
                    format!("{id}.SingleInstance")
                };

                match zbus::blocking::connection::Builder::session()
                    .unwrap()
                    .name(dbus_name.as_str())
                    .unwrap()
                    .replace_existing_names(false)
                    .allow_name_replacements(false)
                    .serve_at(dbus_path.as_str(), single_instance_dbus)
                    .unwrap()
                    .build()
                {
                    Ok(connection) => {
                        app.manage(ConnectionHandle(connection));
                    }
                    Err(zbus::Error::NameTaken) => {
                        if let Ok(connection) = Connection::session() {
                            let _ = connection.call_method(
                                Some(dbus_name.as_str()),
                                dbus_path.as_str(),
                                Some("org.SingleInstance.DBus"),
                                "ExecuteCallback",
                                &(
                                    std::env::args().collect::<Vec<String>>(),
                                    std::env::current_dir()
                                        .unwrap_or_default()
                                        .to_str()
                                        .unwrap_or_default(),
                                ),
                            );
                        }
                        app.cleanup_before_exit();
                        std::process::exit(0);
                    }
                    _ => {}
                }

                Ok(())
            })
            .on_event(move |app, event| {
                if let RunEvent::Exit = event {
                    destroy(app, custom_dbus_id.clone());
                }
            })
            .build()
    }
}
