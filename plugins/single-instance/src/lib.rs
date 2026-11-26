// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Ensure a single instance of your tauri app is running.
//!
//! ## Cargo features
//!
//! - **semver**: Allows the app with SemVer incompatible versions to run alongside each other.
//! - **deep-link**: Trigger [`tauri-plugin-deep-link`](https://crates.io/crates/tauri-plugin-deep-link) event before invoking the single-instance callback.
//!
//! ## Linux
//!
//! The plugin owns a name on the D-Bus session bus, and a second instance calls a method on it.
//! The callback runs inside that method call, on the thread that serves the plugin's D-Bus
//! connection, and the second instance waits for it to return before it exits.
//!
//! In the callback you can use the app handle as usual (emit events, show and focus windows, use
//! state, show a notification with `tauri-plugin-notification`), spawn tasks and threads, and
//! call [`tauri::async_runtime::block_on`].
//!
//! In the callback you can't:
//!
//! - call a blocking API built on `zbus`, such as `zbus::block_on` or `notify-rust`'s
//!   `Notification::show`. It panics, the second instance gets no answer and keeps waiting.
//! - call [`destroy`]. It waits for a reply that only the thread it is on can read, so it never
//!   returns.
//!
//! Both happen to work when some crate in the app enables zbus's `tokio` feature (`ashpd` does by
//! default), because the connection then runs on Tauri's async runtime. Don't rely on that: do
//! such work on another thread instead, which works in every build.
//!
//! ```
//! fn plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
//!     tauri_plugin_single_instance::init(|app, _argv, _cwd| {
//!         let app = app.clone();
//!         std::thread::spawn(move || {
//!             // blocking D-Bus calls are fine here, and so is releasing the lock
//!             tauri_plugin_single_instance::destroy(&app);
//!         });
//!     })
//! }
//! ```
//!
//! The second instance then exits as soon as the callback returns, without waiting for that work.
//! There is no way to keep it waiting for work that has to happen off the callback's thread.
//!
//! Two more limits:
//!
//! - When zbus's `tokio` feature is enabled and you pass a current-thread runtime to
//!   [`tauri::async_runtime::set`], a thread of its own must drive that runtime. If the thread
//!   that builds the app drives it, as under `#[tokio::main(flavor = "current_thread")]`, the app
//!   hangs while the plugin is set up. Use a multi-thread runtime, or keep Tauri's default one.
//!   Don't call [`destroy`] from a task of such a runtime either.
//! - A session bus with a `unixexec:` address (one reached through a helper program such as
//!   `systemd-stdio-bridge`) is not supported. The plugin logs a warning and every instance starts
//!   normally, as when there is no session bus.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use tauri::{AppHandle, Manager, Resource, ResourceId, Runtime, plugin::TauriPlugin};

#[cfg(target_os = "windows")]
#[path = "platform_impl/windows.rs"]
mod platform_impl;
#[cfg(target_os = "linux")]
#[path = "platform_impl/linux.rs"]
mod platform_impl;
#[cfg(target_os = "macos")]
#[path = "platform_impl/macos.rs"]
mod platform_impl;

#[cfg(feature = "semver")]
mod semver_compat;

pub(crate) type SingleInstanceCallback<R> =
    dyn FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static;

/// Initializes the plugin, calling `f` whenever a second instance of the app is started.
///
/// This is a shortcut for [`Builder::new`] with [`Builder::callback`] set to `f`, then
/// [`Builder::build`]. Use [`Builder`] directly if you need to set a custom [`Builder::dbus_id`].
///
/// `f` is called with the app handle, the second instance's command line arguments
/// (as collected by [`std::env::args_os`], so the first element is the executable path) and its
/// current working directory. Arguments and the working directory that are not valid Unicode
/// have their invalid parts replaced by U+FFFD. If the `deep-link` feature is enabled, the arguments are first
/// forwarded to [`tauri-plugin-deep-link`](https://crates.io/crates/tauri-plugin-deep-link)
/// before `f` runs.
///
/// On Linux `f` must not call blocking `zbus`-based APIs or [`destroy`]; see the
/// [crate documentation](crate#linux).
///
/// The second instance never reaches [`tauri::Builder::run`]: it hands its arguments and working
/// directory off to the first instance and exits immediately.
pub fn init<R: Runtime, F: FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static>(
    f: F,
) -> TauriPlugin<R> {
    Builder::new().callback(f).build()
}

/// Releases the resources this plugin uses to detect other instances (the named mutex on
/// Windows, the D-Bus name on Linux or the Unix socket on macOS).
///
/// The plugin releases them automatically in [`AppHandle::cleanup_before_exit`], which Tauri runs
/// right before the process exits or restarts (after your [`tauri::RunEvent::Exit`] handler
/// returns), so you normally don't need to call this yourself and the app keeps being the single
/// instance while your own exit cleanup runs. Call it manually to release the lock earlier, or
/// before terminating the process through means that skip [`AppHandle::cleanup_before_exit`], such
/// as [`std::process::exit`], so a future instance of the app isn't mistaken for a still-running
/// one.
///
/// On Linux, don't call this from the single-instance callback, where it never returns. Spawn a
/// thread for it there; see the [crate documentation](crate#linux).
pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M) {
    if let Some(lock) = manager.try_state::<InstanceLockId>() {
        let _ = manager.resources_table().close(lock.0);
    }
}

struct InstanceLockId(ResourceId);

/// Stores the platform lock in the app's resource table, which Tauri clears in
/// [`AppHandle::cleanup_before_exit`], so the lock's [`Drop`] implementation releases it only once
/// the app is really exiting.
pub(crate) fn manage_instance_lock<R: Runtime, M: Manager<R>>(manager: &M, lock: impl Resource) {
    let rid = manager.resources_table().add(lock);
    manager.manage(InstanceLockId(rid));
}

/// Builds the single-instance plugin.
///
/// Created with [`Builder::new`] and consumed by [`Builder::build`].
pub struct Builder<R: Runtime> {
    callback: Box<SingleInstanceCallback<R>>,
    dbus_id: Option<String>,
}

impl<R: Runtime> Default for Builder<R> {
    fn default() -> Self {
        Self {
            callback: Box::new(move |_app, _args, _| {
                #[cfg(feature = "deep-link")]
                if let Some(deep_link) = _app.try_state::<tauri_plugin_deep_link::DeepLink<R>>() {
                    deep_link.handle_cli_arguments(_args.iter());
                }
            }),
            dbus_id: None,
        }
    }
}

impl<R: Runtime> Builder<R> {
    /// Creates a new builder with a no-op callback (or, when the `deep-link` feature is enabled,
    /// a callback that only forwards the arguments to the deep-link plugin) and no custom D-Bus
    /// ID. Use [`Builder::callback`] and [`Builder::dbus_id`] to configure it, then
    /// [`Builder::build`] to create the plugin.
    pub fn new() -> Self {
        Default::default()
    }

    /// Function to call when a secondary instance was opened by the user and killed by the plugin.
    /// If the `deep-link` feature is enabled, the plugin triggers the deep-link plugin before executing the callback.
    ///
    /// On Linux the callback must not call blocking `zbus`-based APIs or [`destroy`]; see the
    /// [crate documentation](crate#linux).
    pub fn callback<F: FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static>(
        mut self,
        mut f: F,
    ) -> Self {
        self.callback = Box::new(move |app, args, cwd| {
            #[cfg(feature = "deep-link")]
            if let Some(deep_link) = app.try_state::<tauri_plugin_deep_link::DeepLink<R>>() {
                deep_link.handle_cli_arguments(args.iter());
            }
            f(app, args, cwd)
        });
        self
    }

    /// Set a custom D-Bus ID, used on Linux. The plugin will append a `.SingleInstance` subname.
    /// For example `com.mycompany.myapp` will result in the plugin registering its D-Bus service on `com.mycompany.myapp.SingleInstance`.
    /// Usually you want the same base ID across all components in your app.
    ///
    /// Defaults to the app's bundle identifier set in tauri.conf.json.
    pub fn dbus_id(mut self, dbus_id: impl Into<String>) -> Self {
        self.dbus_id = Some(dbus_id.into());
        self
    }

    /// Builds the single-instance [`TauriPlugin`].
    ///
    /// Register it first among your app's plugins, since the plugins run in the order they were
    /// added and a second instance must be detected before the rest of your setup runs.
    pub fn build(self) -> TauriPlugin<R> {
        platform_impl::init(
            self.callback,
            #[cfg(target_os = "linux")]
            self.dbus_id,
        )
    }
}
