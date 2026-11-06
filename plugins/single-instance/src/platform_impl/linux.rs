// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;

use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
};

use crate::{SingleInstanceCallback, manage_instance_lock};
use tauri::{
    AppHandle, Resource, Runtime,
    plugin::{self, TauriPlugin},
};
use tokio::sync::oneshot;
use zbus::{Connection, ObjectPath, connection, interface, names::WellKnownName};

/// Owns the D-Bus name, released when the app exits (see [`crate::destroy`]).
struct InstanceLock {
    shutdown: Option<oneshot::Sender<()>>,
    thread: Option<JoinHandle<()>>,
}

impl Resource for InstanceLock {}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        // The bus thread releases the name and exits once told to.
        if let Some(shutdown) = self.shutdown.take() {
            let _ = shutdown.send(());
        }
        // The callback runs on the bus thread, which can't wait for itself.
        if let Some(thread) = self.thread.take()
            && thread.thread().id() != thread::current().id()
        {
            let _ = thread.join();
        }
    }
}

/// How the bus thread's attempt to own the D-Bus name ended.
enum Startup {
    /// This is the first instance, and the thread now serves [`SingleInstanceDBus`].
    Primary,
    /// Another instance owns the name and accepted this one's arguments.
    Forwarded,
    /// Another instance owns the name, but the arguments didn't reach it.
    ForwardFailed(zbus::Error),
    /// The session bus can't be used.
    Unavailable(zbus::Error),
}

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

/// Hands this process's arguments and working directory to the instance that owns `name`.
async fn forward_to_primary(name: &WellKnownName<'_>, path: &ObjectPath<'_>) -> zbus::Result<()> {
    let connection = Connection::session().await?;
    connection
        .call_method(
            Some(name),
            path,
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
        .await?;
    Ok(())
}

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

            // Checked here so that an invalid `dbus_id` fails setup rather than reading as a
            // missing bus, since the connection builder reports both from `build()`.
            let name = WellKnownName::try_from(dbus_name)?;
            let path = ObjectPath::try_from(dbus_path)?;

            let single_instance_dbus = SingleInstanceDBus {
                callback,
                app_handle: app.clone(),
            };

            // With zbus's `tokio` feature, which any crate in the app can turn on, a connection
            // built in `zbus::block_on` is only served while a thread is inside that call. So the
            // first instance's connection stays in `block_on` on a thread of its own until the
            // lock is dropped.
            let (startup_tx, startup_rx) = mpsc::channel();
            let (shutdown_tx, shutdown_rx) = oneshot::channel();
            let thread = thread::Builder::new()
                .name("single-instance-dbus".into())
                .spawn(move || {
                    zbus::block_on(async move {
                        let startup = match connection::Builder::session()
                            .name(&name)
                            .replace_existing_names(false)
                            .allow_name_replacements(false)
                            .serve_at(&path, single_instance_dbus)
                            .build()
                            .await
                        {
                            Ok(connection) => {
                                let _ = startup_tx.send(Startup::Primary);
                                let _ = shutdown_rx.await;
                                let _ = connection.release_name(&name).await;
                                return;
                            }
                            Err(zbus::Error::NameTaken) => {
                                match forward_to_primary(&name, &path).await {
                                    Ok(()) => Startup::Forwarded,
                                    Err(error) => Startup::ForwardFailed(error),
                                }
                            }
                            Err(error) => Startup::Unavailable(error),
                        };
                        let _ = startup_tx.send(startup);
                    })
                })?;

            match startup_rx.recv() {
                Ok(Startup::Primary) => {
                    manage_instance_lock(
                        app,
                        InstanceLock {
                            shutdown: Some(shutdown_tx),
                            thread: Some(thread),
                        },
                    );
                }
                Ok(Startup::Forwarded) => {
                    // Exit successfully only after the primary accepted the arguments.
                    app.cleanup_before_exit();
                    std::process::exit(0);
                }
                Ok(Startup::ForwardFailed(error)) => {
                    return Err(format!(
                        "failed to forward arguments to the running instance: {error}"
                    )
                    .into());
                }
                Ok(Startup::Unavailable(error)) => {
                    // Without a session bus there is no way to find other instances.
                    tracing::warn!(
                        "single-instance: D-Bus session bus unavailable, launching normally: {error}"
                    );
                }
                Err(_) => return Err("single-instance: the D-Bus thread exited unexpectedly".into()),
            }

            Ok(())
        })
        .build()
}
