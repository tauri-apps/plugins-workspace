// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Ensure a single instance of your tauri app is running.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg(not(any(target_os = "android", target_os = "ios")))]

use tauri::{plugin::TauriPlugin, AppHandle, Manager, Runtime};

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

pub fn init<R: Runtime, F: FnMut(&AppHandle<R>, Vec<String>, String) + Send + Sync + 'static>(
    mut f: F,
) -> TauriPlugin<R> {
    platform_impl::init(Box::new(move |app, args, cwd| {
        #[cfg(feature = "deep-link")]
        if let Some(deep_link) = app.try_state::<tauri_plugin_deep_link::DeepLink<R>>() {
            deep_link.handle_cli_arguments(args.iter());
        }
        f(app, args, cwd)
    }))
}

pub fn destroy<R: Runtime, M: Manager<R>>(manager: &M) {
    platform_impl::destroy(manager)
}
