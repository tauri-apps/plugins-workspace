// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// Every CEF application is also its own renderer, GPU, network and utility
// process. This attribute runs the helper side of that and returns before the
// Tauri application is built, for any process Chromium launched with `--type=`.
#[cfg_attr(feature = "cef", tauri_runtime_cef::cef_entry_point)]
fn main() {
    #[cfg(feature = "cef")]
    let builder = tauri::Builder::default().runtime(tauri_runtime_cef::Cef::default());
    #[cfg(not(feature = "cef"))]
    let builder = tauri::Builder::default().runtime(tauri_runtime_wry::Wry::default());

    builder
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
