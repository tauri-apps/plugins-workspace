// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::Listener;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, argv, _cwd| {
            println!("single instance triggered: {argv:?}");
        }))
        .plugin(tauri_plugin_deep_link::init())
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            // ensure deep links are registered on the system
            // this is useful because AppImages requires additional setup to be available in the system
            // and calling register() makes the deep links immediately available - without any user input
            #[cfg(target_os = "linux")]
            {
                use tauri_plugin_deep_link::DeepLinkExt;

                let schemes = app
                    .config()
                    .plugins
                    .0
                    .get("deep-link")
                    .unwrap()
                    .get("desktop")
                    .unwrap()
                    .get("schemes")
                    .unwrap()
                    .as_array()
                    .unwrap();

                for scheme in schemes {
                    app.deep_link().register(scheme.as_str().unwrap())?;
                }
            }

            app.listen("deep-link://new-url", |url| {
                dbg!(url);
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
