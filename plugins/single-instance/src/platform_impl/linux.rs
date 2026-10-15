// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;

use crate::SingleInstanceCallback;
use tauri::{
    AppHandle, Manager, RunEvent, Runtime,
    plugin::{self, TauriPlugin},
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

pub fn init<R: Runtime>(
    callback: Box<SingleInstanceCallback<R>>,
    dbus_id: Option<String>,
) -> TauriPlugin<R> {
    plugin::Builder::new("single-instance")
        .setup(move |app, _api| {
            let mut dbus_name = dbus_id.unwrap_or_else(|| app.config().identifier.clone());
            dbus_name.push_str(".SingleInstance");

            #[cfg(feature = "semver")]
            {
                dbus_name.push('_');
                dbus_name.push_str(semver_compat_string(&app.package_info().version).as_str());
            }

            let mut dbus_path = dbus_name.replace('.', "/").replace('-', "_");
            if !dbus_path.starts_with('/') {
                dbus_path = format!("/{dbus_path}");
            }

            let single_instance_dbus = SingleInstanceDBus {
                callback,
                app_handle: app.clone(),
            };

            let builder = match zbus::blocking::connection::Builder::session() {
                Ok(builder) => builder,
                Err(error) => {
                    tracing::warn!(
                        "single-instance: invalid D-Bus session address, launching normally: {error}"
                    );
                    return Ok(());
                }
            };

            match builder
                .name(dbus_name.as_str())?
                .replace_existing_names(false)
                .allow_name_replacements(false)
                .serve_at(dbus_path.as_str(), single_instance_dbus)?
                .build()
            {
                Ok(connection) => {
                    app.manage(ConnectionHandle(connection));
                }
                Err(zbus::Error::NameTaken) => {
                    let connection = Connection::session()?;
                    connection
                        .call_method(
                            Some(dbus_name.as_str()),
                            dbus_path.as_str(),
                            Some("org.SingleInstance.DBus"),
                            "ExecuteCallback",
                            &(
                                std::env::args_os()
                                    .map(|arg| arg.to_string_lossy().into_owned())
                                    .collect::<Vec<String>>(),
                                std::env::current_dir()
                                    .unwrap_or_default()
                                    .to_string_lossy()
                                    .as_ref(),
                            ),
                        )
                        .map_err(|error| {
                            format!("failed to forward arguments to the running instance: {error}")
                        })?;
                    // Exit successfully only after the primary accepted the arguments.
                    app.cleanup_before_exit();
                    std::process::exit(0);
                }
                Err(error) => {
                    // Without a session bus there is no way to find other instances.
                    tracing::warn!(
                        "single-instance: D-Bus session bus unavailable, launching normally: {error}"
                    );
                    return Ok(());
                }
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
    if let Some(connection) = manager.try_state::<ConnectionHandle>()
        && let Some(dbus_name) = manager
            .try_state::<DBusName>()
            .and_then(|name| WellKnownName::try_from(name.0.clone()).ok())
    {
        let _ = connection.0.release_name(dbus_name);
    }
}
