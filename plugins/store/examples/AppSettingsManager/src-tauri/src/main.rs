// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use serde_json::json;
use tauri::Listener;
use tauri_plugin_store::StoreExt;

mod app;
use app::settings::AppSettings;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .setup(|app| {
            // Init store and load it from disk
            let store = app.store("settings.json")?;
            app.listen("store://change", |event| {
                dbg!(event);
            });
            let app_settings = AppSettings::load_from_store(&store);
            match app_settings {
                Ok(app_settings) => {
                    let theme = app_settings.theme;
                    let launch_at_login = app_settings.launch_at_login;

                    println!("theme {theme}");
                    println!("launch_at_login {launch_at_login}");
                    store.set(
                        "appSettings",
                        json!({ "theme": theme, "launchAtLogin": launch_at_login }),
                    );
                }
                Err(err) => {
                    eprintln!("Error loading settings: {err}");
                    // Handle the error case if needed
                    return Err(err); // Convert the error to a Box<dyn Error> and return Err(err) here
                }
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
