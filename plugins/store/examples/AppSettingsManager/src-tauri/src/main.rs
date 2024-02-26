// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use tauri_plugin_store::StoreBuilder;

mod app;
use app::settings::AppSettings;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(|app| {
            //start settings store
            let store = StoreBuilder::new(app.handle(), "settings.json".parse()?).build();
            let store_arc = Arc::new(store);
            let app_settings_result = AppSettings::load_from_store(&store_arc); // Use store_arc here

            match app_settings_result {
                Ok(app_settings) => {
                    let theme = app_settings.theme;
                    let launch_at_login = app_settings.launch_at_login;

                    eprintln!("theme {}", theme);
                    eprintln!("launch_at_login {}", launch_at_login);

                    // Define 'launch_at_login' or use a proper condition here
                    if launch_at_login {
                        eprintln!("Launch at login TRUE");
                        // Handle the case where launch_at_login is true
                    } else {
                        eprintln!("Launch at login FALSE");
                        // Handle the case where launch_at_login is false
                    }
                    Ok(()) // Return Ok(()) here
                }
                Err(err) => {
                    eprintln!("Error loading settings: {}", err);
                    // Handle the error case if needed
                    Err(err.into()) // Convert the error to a Box<dyn Error> and return Err(err) here
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
