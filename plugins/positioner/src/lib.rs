// Copyright 2021 Jonas Kruckenberg
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! [![](https://github.com/tauri-apps/plugins-workspace/raw/v2/plugins/positioner/banner.png)](https://github.com/tauri-apps/plugins-workspace/tree/v2/plugins/positioner)
//!
//! A plugin for Tauri that helps position your windows at well-known locations.
//!
//! # Cargo features
//!
//! - **tray-icon**: Enables tray-icon-relative positions.
//!
//!   Note: This requires attaching the Tauri plugin, *even* when using the trait extension only.

#![doc(
    html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png",
    html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/app-icon.png"
)]
#![cfg(not(any(target_os = "android", target_os = "ios")))]

mod ext;

pub use ext::*;
use tauri::{
    plugin::{self, TauriPlugin},
    Result, Runtime,
};

#[cfg(feature = "tray-icon")]
use tauri::{tray::TrayIconEvent, AppHandle, Manager, PhysicalPosition, PhysicalSize};

#[cfg(feature = "tray-icon")]
struct Tray(std::sync::Mutex<Option<(PhysicalPosition<f64>, PhysicalSize<f64>)>>);

#[cfg(feature = "tray-icon")]
pub fn on_tray_event<R: Runtime>(app: &AppHandle<R>, event: &TrayIconEvent) {
    let position = PhysicalPosition {
        x: event.x,
        y: event.y,
    };
    let size = PhysicalSize {
        width: event.icon_rect.right - event.icon_rect.left,
        height: event.icon_rect.bottom - event.icon_rect.top,
    };
    app.state::<Tray>()
        .0
        .lock()
        .unwrap()
        .replace((position, size));
}

#[tauri::command]
async fn move_window<R: Runtime>(
    webview_window: tauri::WebviewWindow<R>,
    position: Position,
) -> Result<()> {
    webview_window.move_window(position)
}

/// The Tauri plugin that exposes [`WindowExt::move_window`] to the webview.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let plugin = plugin::Builder::new("positioner")
        .js_init_script(include_str!("api-iife.js").to_string())
        .invoke_handler(tauri::generate_handler![move_window]);

    #[cfg(feature = "tray-icon")]
    let plugin = plugin.setup(|app_handle, _api| {
        app_handle.manage(Tray(std::sync::Mutex::new(None)));
        Ok(())
    });

    plugin.build()
}
