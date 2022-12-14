// Copyright 2021 Jonas Kruckenberg
// SPDX-License-Identifier: MIT

//! A plugin for Tauri that helps position your windows at well-known locations.
//!
//! # Cargo features
//!
//! - **system-tray**: Enables system-tray-relative positions.
//!   
//!   Note: This requires attaching the Tauri plugin, *even* when using the trait extension only.

mod ext;

pub use ext::*;
use tauri::{
    plugin::{self, TauriPlugin},
    Result, Runtime,
};

#[cfg(feature = "system-tray")]
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, SystemTrayEvent};

#[cfg(feature = "system-tray")]
struct Tray(std::sync::Mutex<Option<(PhysicalPosition<f64>, PhysicalSize<f64>)>>);

#[cfg(feature = "system-tray")]
pub fn on_tray_event<R: Runtime>(app: &AppHandle<R>, event: &SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { position, size, .. } => {
            app.state::<Tray>()
                .0
                .lock()
                .unwrap()
                .replace((*position, *size));
        }
        SystemTrayEvent::RightClick { position, size, .. } => {
            app.state::<Tray>()
                .0
                .lock()
                .unwrap()
                .replace((*position, *size));
        }
        SystemTrayEvent::DoubleClick { position, size, .. } => {
            app.state::<Tray>()
                .0
                .lock()
                .unwrap()
                .replace((*position, *size));
        }
        _ => (),
    }
}

#[tauri::command]
async fn move_window<R: Runtime>(window: tauri::Window<R>, position: Position) -> Result<()> {
    window.move_window(position)
}

/// The Tauri plugin that exposes [`WindowExt::move_window`] to the webview.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    let plugin =
        plugin::Builder::new("positioner").invoke_handler(tauri::generate_handler![move_window]);

    #[cfg(feature = "system-tray")]
    let plugin = plugin.setup(|app_handle| {
        app_handle.manage(Tray(std::sync::Mutex::new(None)));
        Ok(())
    });

    plugin.build()
}
