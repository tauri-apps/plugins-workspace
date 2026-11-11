// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;

use std::{future::Future, thread};

use crate::{SingleInstanceCallback, manage_instance_lock};
use tauri::{
    AppHandle, Resource, Runtime,
    plugin::{self, TauriPlugin},
};
use tokio::runtime::{Handle, RuntimeFlavor};
use zbus::{Connection, ObjectPath, connection, interface, names::WellKnownName};

/// Runs `future` on Tauri's async runtime, so that a connection built in it keeps being served
/// there after this returns. The short-lived thread is there because
/// [`tauri::async_runtime::block_on`] panics on a thread that is already inside a Tokio runtime.
fn block_on<F: Future + Send>(future: F) -> F::Output
where
    F::Output: Send,
{
    thread::scope(|scope| {
        let thread = scope.spawn(|| tauri::async_runtime::block_on(future));
        // The future may need the runtime's workers, such as for the task that reads the
        // connection's replies, while this thread could be the only one.
        block_in_place(|| thread.join()).unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
}

/// Tells Tokio that `f` may block when this thread is a worker of a multi-thread runtime, which
/// hands the worker's other duties, such as polling for I/O, to another thread in the meantime.
fn block_in_place<T>(f: impl FnOnce() -> T) -> T {
    match Handle::try_current() {
        Ok(handle) if handle.runtime_flavor() == RuntimeFlavor::MultiThread => {
            tokio::task::block_in_place(f)
        }
        _ => f(),
    }
}

/// Owns the D-Bus name, released when the app exits (see [`crate::destroy`]).
struct InstanceLock {
    connection: Connection,
    name: WellKnownName<'static>,
}

impl Resource for InstanceLock {}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        let _ = block_on(self.connection.release_name(&self.name));
    }
}

struct SingleInstanceDBus<R: Runtime> {
    callback: Box<SingleInstanceCallback<R>>,
    app_handle: AppHandle<R>,
}

#[interface(name = "org.SingleInstance.DBus")]
impl<R: Runtime> SingleInstanceDBus<R> {
    fn execute_callback(&mut self, argv: Vec<String>, cwd: String) {
        // The callback runs in a task on Tauri's runtime.
        block_in_place(|| (self.callback)(&self.app_handle, argv, cwd));
    }
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

            match block_on(
                connection::Builder::session()
                    .name(&name)
                    .replace_existing_names(false)
                    .allow_name_replacements(false)
                    .serve_at(&path, single_instance_dbus)
                    .build(),
            ) {
                Ok(connection) => {
                    manage_instance_lock(app, InstanceLock { connection, name });
                }
                Err(zbus::Error::NameTaken) => {
                    block_on(async {
                        Connection::session()
                            .await?
                            .call_method(
                                Some(&name),
                                &path,
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
                            .await
                    })
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

            Ok(())
        })
        .build()
}
