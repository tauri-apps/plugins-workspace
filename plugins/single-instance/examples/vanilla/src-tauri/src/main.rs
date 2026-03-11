// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg(feature = "cef")]
type TauriRuntime = tauri_runtime_cef::CefRuntime<tauri::EventLoopMessage>;
#[cfg(feature = "wry")]
type TauriRuntime = tauri_runtime_wry::Wry<tauri::EventLoopMessage>;

fn main() {
    tauri::Builder::<TauriRuntime>::new()
        .plugin(
            tauri_plugin_single_instance::Builder::new()
                .callback(move |app, argv, cwd| {
                    println!("{}, {argv:?}, {cwd}", app.package_info().name);
                })
                .dbus_id("org.Tauri.SIExampleApp".to_owned())
                .build(),
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
