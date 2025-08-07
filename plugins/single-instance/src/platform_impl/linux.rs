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
use zbus::{
    blocking::{connection::Builder, Connection},
    interface,
};

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
    config.identifier.replace(['.', '-'], "_")
}

pub fn init<R: Runtime>(f: Box<SingleInstanceCallback<R>>) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(|app, _api| {
            #[cfg(feature = "semver")]
            let id = dbus_id(app.config(), app.package_info().version.clone());
            #[cfg(not(feature = "semver"))]
            let id = dbus_id(app.config());

            let single_instance_dbus = SingleInstanceDBus {
                callback: f,
                app_handle: app.clone(),
            };
            let dbus_name = format!("org.{id}.SingleInstance");
            let dbus_path = format!("/org/{id}/SingleInstance");

            match Builder::session()
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
        .on_event(|app, event| {
            if let RunEvent::Exit = event {
                destroy(app);
            }
        })
        .build()
}

pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M) {
    if let Some(connection) = manager.try_state::<ConnectionHandle>() {
        #[cfg(feature = "semver")]
        let id = dbus_id(
            manager.config(),
            manager.app_handle().package_info().version.clone(),
        );
        #[cfg(not(feature = "semver"))]
        let id = dbus_id(manager.config());

        let dbus_name = format!("org.{id}.SingleInstance",);
        let _ = connection.0.release_name(dbus_name);
    }
}
