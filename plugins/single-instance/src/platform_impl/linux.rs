// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;

use crate::SingleInstanceCallback;
use tauri::{
    plugin::{self, TauriPlugin},
    AppHandle, Config, Manager, RunEvent, Runtime,
};
use zbus::{blocking::Connection, interface, names::WellKnownName};

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

struct DBusName(String);

#[cfg(feature = "semver")]
fn default_dbus_id(config: &Config, version: semver::Version) -> String {
    let mut id = config.identifier.replace(['.', '-'], "_");
    id.push('_');
    id.push_str(semver_compat_string(version).as_str());
    id
}

#[cfg(not(feature = "semver"))]
fn default_dbus_id(config: &Config) -> String {
    config.identifier.clone()
}

fn dbus_path(dbus_name: &str) -> String {
    dbus_name.replace('.', "/")
}

pub fn init<R: Runtime>(
    callback: Box<SingleInstanceCallback<R>>,
    dbus_id: Option<String>,
) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(move |app, _api| {
            let dbus_name = if let Some(dbus_id) = dbus_id {
                dbus_id
            } else {
                let id = default_dbus_id(
                    app.config(),
                    #[cfg(feature = "semver")]
                    app.package_info().version.clone(),
                );
                format!("{id}.SingleInstance")
            };
            let dbus_path = dbus_path(&dbus_name);

            let single_instance_dbus = SingleInstanceDBus {
                callback,
                app_handle: app.clone(),
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

            app.manage(DBusName(dbus_name));

            Ok(())
        })
        .on_event(move |app, event| {
            if let RunEvent::Exit = event {
                destroy(app);
            }
        })
        .build()
}

pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M) {
    if let Some(connection) = manager.try_state::<ConnectionHandle>() {
        if let Some(dbus_name) = manager
            .try_state::<DBusName>()
            .and_then(|name| WellKnownName::try_from(name.0.clone()).ok())
        {
            let _ = connection.0.release_name(dbus_name);
        }
    }
}
