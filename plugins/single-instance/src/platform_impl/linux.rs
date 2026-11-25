// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(feature = "semver")]
use crate::semver_compat::semver_compat_string;

use std::{
    future::Future,
    panic::{AssertUnwindSafe, catch_unwind},
    pin::pin,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    task::{Context, Poll, Wake, Waker},
    thread::{self, Thread},
};

use crate::{SingleInstanceCallback, manage_instance_lock};
use tauri::{
    Resource, Runtime,
    plugin::{self, TauriPlugin},
};
use tokio::sync::oneshot;
use zbus::{Connection, ObjectPath, connection, interface, names::WellKnownName};

/// Wakes the thread parked in [`block_on`].
struct Unpark {
    thread: Thread,
    woken: AtomicBool,
}

impl Wake for Unpark {
    fn wake(self: Arc<Self>) {
        self.woken.store(true, Ordering::Release);
        self.thread.unpark();
    }
}

/// Polls `future` to completion on a short-lived thread that is in no runtime. zbus picks the
/// runtime of a connection from the thread that builds it, so this way it always serves ours on
/// its own runtime, whatever zbus features the rest of the app enables and whatever runtime the
/// caller is part of. The wait therefore never needs the calling thread to make progress.
fn block_on<F: Future + Send>(future: F) -> F::Output
where
    F::Output: Send,
{
    thread::scope(|scope| {
        scope
            .spawn(|| {
                let unpark = Arc::new(Unpark {
                    thread: thread::current(),
                    woken: AtomicBool::new(false),
                });
                let waker = Waker::from(unpark.clone());
                let mut cx = Context::from_waker(&waker);
                let mut future = pin!(future);
                loop {
                    if let Poll::Ready(output) = future.as_mut().poll(&mut cx) {
                        return output;
                    }
                    while !unpark.woken.swap(false, Ordering::Acquire) {
                        thread::park();
                    }
                }
            })
            .join()
            .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
    })
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

type Call = (Vec<String>, String, oneshot::Sender<()>);

struct SingleInstanceDBus {
    calls: mpsc::Sender<Call>,
}

#[interface(name = "org.SingleInstance.DBus")]
impl SingleInstanceDBus {
    async fn execute_callback(&self, argv: Vec<String>, cwd: String) -> zbus::fdo::Result<()> {
        // The callback runs on the plugin's own thread, never on the one serving the connection,
        // so it may block, including on D-Bus calls and on `destroy`.
        let (done, wait) = oneshot::channel();
        let failed = || zbus::fdo::Error::Failed("the single-instance callback panicked".into());
        self.calls.send((argv, cwd, done)).map_err(|_| failed())?;
        wait.await.map_err(|_| failed())
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

            let (calls, pending) = mpsc::channel::<Call>();
            let single_instance_dbus = SingleInstanceDBus { calls };
            let app_handle = app.clone();
            let mut callback = callback;
            thread::Builder::new()
                .name("single-instance".into())
                .spawn(move || {
                    for (argv, cwd, done) in pending {
                        // A panic drops `done`, which fails that second instance only.
                        let call = AssertUnwindSafe(|| callback(&app_handle, argv, cwd));
                        if catch_unwind(call).is_ok() {
                            let _ = done.send(());
                        }
                    }
                })?;

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
