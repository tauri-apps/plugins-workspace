// Copyright 2021 Jonas Kruckenberg
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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
    let (position, size) = {
        match event {
            TrayIconEvent::Click { rect, .. }
            | TrayIconEvent::Enter { rect, .. }
            | TrayIconEvent::Leave { rect, .. }
            | TrayIconEvent::Move { rect, .. } => {
                // tray-icon emits PhysicalSize so the scale factor should not matter.
                let size = rect.size.to_physical(1.0);
                let position = rect.position.to_physical(1.0);
                (position, size)
            }

            _ => return,
        }
    };

    app.state::<Tray>()
        .0
        .lock()
        .unwrap()
        .replace((position, size));
}

#[tauri::command]
async fn move_window<R: Runtime>(window: tauri::Window<R>, position: Position) -> Result<()> {
    window.move_window(position)
}

#[cfg(feature = "tray-icon")]
#[tauri::command]
async fn move_window_constrained<R: Runtime>(
    window: tauri::Window<R>,
    position: Position,
) -> Result<()> {
    window.move_window_constrained(position)
}

#[cfg(feature = "tray-icon")]
#[tauri::command]
fn set_tray_icon_state<R: Runtime>(
    app: AppHandle<R>,
    position: PhysicalPosition<f64>,
    size: PhysicalSize<f64>,
) {
    app.state::<Tray>()
        .0
        .lock()
        .unwrap()
        .replace((position, size));
}

/// The Tauri plugin that exposes [`WindowExt::move_window`] to the webview.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let plugin = plugin::Builder::new("positioner").invoke_handler(tauri::generate_handler![
        move_window,
        #[cfg(feature = "tray-icon")]
        move_window_constrained,
        #[cfg(feature = "tray-icon")]
        set_tray_icon_state
    ]);

    #[cfg(feature = "tray-icon")]
    let plugin = plugin.setup(|app_handle, _api| {
        app_handle.manage(Tray(std::sync::Mutex::new(None)));
        Ok(())
    });

    plugin.build()
}
