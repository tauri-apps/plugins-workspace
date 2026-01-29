// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

fn main() {
    let mut builder = tauri::Builder::default();

    #[cfg(not(target_os = "linux"))]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            println!("{}, {argv:?}, {cwd}", app.package_info().name);
        }));
    }
    #[cfg(target_os = "linux")]
    {
        let single_instance = tauri_plugin_single_instance::Builder::new()
            .dbus_id("org.Example.app".to_owned())
            .build(Box::new(move |app, argv, cwd| {
                println!("{}, {argv:?}, {cwd}", app.package_info().name);
            }));
        builder = builder.plugin(single_instance);
    }

    builder
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
