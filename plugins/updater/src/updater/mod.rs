// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Tauri updater.
//!
//! The updater is focused on making Tauri's application updates **as safe and transparent as updates to a website**.
//!
//! For a full guide on setting up the updater, see <https://tauri.app/v1/guides/distribution/updater>.
//!
//! Check [`UpdateBuilder`] to see how to trigger and customize the updater at runtime.
//! ```

mod core;
mod extract;
mod move_file;

use std::time::Duration;

use http::header::{HeaderName, HeaderValue};
use semver::Version;
use time::OffsetDateTime;

pub use self::core::{DownloadEvent, RemoteRelease};

use tauri::{AppHandle, Manager, Runtime};

use crate::{Result, UpdaterState};

/// Gets the target string used on the updater.
pub fn target() -> Option<String> {
    if let (Some(target), Some(arch)) = (core::get_updater_target(), core::get_updater_arch()) {
        Some(format!("{target}-{arch}"))
    } else {
        None
    }
}

#[derive(Clone, serde::Serialize)]
struct StatusEvent {
    status: String,
    error: Option<String>,
}

#[derive(Clone, serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadProgressEvent {
    chunk_length: usize,
    content_length: Option<u64>,
}

#[derive(Clone, serde::Serialize)]
struct UpdateManifest {
    version: String,
    date: Option<String>,
    body: String,
}

/// An update check builder.
#[derive(Debug)]
pub struct UpdateBuilder<R: Runtime> {
    inner: core::UpdateBuilder<R>,
}

impl<R: Runtime> UpdateBuilder<R> {
    /// Sets the current platform's target name for the updater.
    ///
    /// The target is injected in the endpoint URL by replacing `{{target}}`.
    /// Note that this does not affect the `{{arch}}` variable.
    ///
    /// If the updater response JSON includes the `platforms` field,
    /// that object must contain a value for the target key.
    ///
    /// By default Tauri uses `$OS_NAME` as the replacement for `{{target}}`
    /// and `$OS_NAME-$ARCH` as the key in the `platforms` object,
    /// where `$OS_NAME` is the current operating system name "linux", "windows" or "darwin")
    /// and `$ARCH` is one of the supported architectures ("i686", "x86_64", "armv7" or "aarch64").
    ///
    /// See [`Builder::updater_target`](crate::Builder#method.updater_target) for a way to set the target globally.
    ///
    /// # Examples
    ///
    /// ## Use a macOS Universal binary target name
    ///
    /// In this example, we set the updater target only on macOS.
    /// On other platforms, we set the default target.
    /// Note that `{{target}}` will be replaced with `darwin-universal`,
    /// but `{{arch}}` is still the running platform's architecture.
    ///
    /// ```no_run
    /// use tauri_plugin_updater::{target as updater_target, UpdaterExt};
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle();
    ///     tauri::async_runtime::spawn(async move {
    ///       let builder = handle.updater().target(if cfg!(target_os = "macos") {
    ///         "darwin-universal".to_string()
    ///       } else {
    ///         updater_target().unwrap()
    ///       });
    ///       match builder.check().await {
    ///         Ok(update) => {}
    ///         Err(error) => {}
    ///       }
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    ///
    /// ## Append debug information to the target
    ///
    /// This allows you to provide updates for both debug and release applications.
    ///
    /// ```no_run
    /// use tauri_plugin_updater::{UpdaterExt, target as updater_target};
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle();
    ///     tauri::async_runtime::spawn(async move {
    ///       let kind = if cfg!(debug_assertions) { "debug" } else { "release" };
    ///       let builder = handle.updater().target(format!("{}-{kind}", updater_target().unwrap()));
    ///       match builder.check().await {
    ///         Ok(update) => {}
    ///         Err(error) => {}
    ///       }
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    ///
    /// ## Use the platform's target triple
    ///
    /// ```no_run
    /// use tauri_plugin_updater::UpdaterExt;
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle();
    ///     tauri::async_runtime::spawn(async move {
    ///       let builder = handle.updater().target(tauri::utils::platform::target_triple().unwrap());
    ///       match builder.check().await {
    ///         Ok(update) => {}
    ///         Err(error) => {}
    ///       }
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.inner = self.inner.target(target);
        self
    }

    /// Sets a closure that is invoked to compare the current version and the latest version returned by the updater server.
    /// The first argument is the current version, and the second one is the latest version.
    ///
    /// The closure must return `true` if the update should be installed.
    ///
    /// # Examples
    ///
    /// - Always install the version returned by the server:
    ///
    /// ```no_run
    /// use tauri_plugin_updater::UpdaterExt;
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     app.handle().updater().should_install(|_current, _latest| true);
    ///     Ok(())
    ///   });
    /// ```
    pub fn should_install<F: FnOnce(&Version, &RemoteRelease) -> bool + Send + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.inner = self.inner.should_install(f);
        self
    }

    /// Sets the timeout for the requests to the updater endpoints.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.inner = self.inner.timeout(timeout);
        self
    }

    /// Add a `Header` to the request.
    pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        self.inner = self.inner.header(key, value)?;
        Ok(self)
    }

    /// Check if an update is available.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use tauri_plugin_updater::{UpdaterExt, DownloadEvent};
    /// tauri::Builder::default()
    ///   .setup(|app| {
    ///     let handle = app.handle();
    ///     tauri::async_runtime::spawn(async move {
    ///       match handle.updater().check().await {
    ///         Ok(update) => {
    ///           if update.is_update_available() {
    ///             update.download_and_install(|event| {
    ///                 match event {
    ///                     DownloadEvent::Started { content_length } => println!("started! size: {:?}", content_length),
    ///                     DownloadEvent::Progress { chunk_length } => println!("Downloaded {chunk_length} bytes"),
    ///                     DownloadEvent::Finished => println!("download finished"),
    ///                 }
    ///             }).await.unwrap();
    ///           }
    ///         }
    ///         Err(e) => {
    ///           println!("failed to get update: {}", e);
    ///         }
    ///       }
    ///     });
    ///     Ok(())
    ///   });
    /// ```
    pub async fn check(self) -> Result<UpdateResponse<R>> {
        self.inner
            .build()
            .await
            .map(|update| UpdateResponse { update })
    }
}

/// The response of an updater check.
pub struct UpdateResponse<R: Runtime> {
    update: core::Update<R>,
}

impl<R: Runtime> Clone for UpdateResponse<R> {
    fn clone(&self) -> Self {
        Self {
            update: self.update.clone(),
        }
    }
}

impl<R: Runtime> UpdateResponse<R> {
    /// Whether the updater found a newer release or not.
    pub fn is_update_available(&self) -> bool {
        self.update.should_update
    }

    /// The current version of the application as read by the updater.
    pub fn current_version(&self) -> &Version {
        &self.update.current_version
    }

    /// The latest version of the application found by the updater.
    pub fn latest_version(&self) -> &str {
        &self.update.version
    }

    /// The update date.
    pub fn date(&self) -> Option<&OffsetDateTime> {
        self.update.date.as_ref()
    }

    /// The update description.
    pub fn body(&self) -> Option<&String> {
        self.update.body.as_ref()
    }

    /// Downloads and installs the update.
    pub async fn download_and_install<F: Fn(DownloadEvent)>(&self, on_event: F) -> Result<()> {
        // Launch updater download process
        // macOS we display the `Ready to restart dialog` asking to restart
        // Windows is closing the current App and launch the downloaded MSI when ready (the process stop here)
        // Linux we replace the AppImage by launching a new install, it start a new AppImage instance, so we're closing the previous. (the process stop here)
        self.update
            .download_and_install(
                self.update.app.config().tauri.bundle.updater.pubkey.clone(),
                on_event,
            )
            .await
    }
}

/// Initializes the [`UpdateBuilder`] using the app configuration.
pub fn builder<R: Runtime>(handle: AppHandle<R>) -> UpdateBuilder<R> {
    let package_info = handle.package_info().clone();

    // prepare our endpoints
    let endpoints = handle
        .state::<UpdaterState>()
        .config
        .endpoints
        .iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>();

    let mut builder = self::core::builder(handle.clone())
        .urls(&endpoints[..])
        .current_version(package_info.version);
    if let Some(target) = &handle.state::<crate::UpdaterState>().target {
        builder = builder.target(target);
    }
    UpdateBuilder { inner: builder }
}
