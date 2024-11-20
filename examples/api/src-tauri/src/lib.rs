// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod cmd;
#[cfg(desktop)]
mod tray;

use serde::Serialize;
use tauri::{
    webview::{PageLoadEvent, WebviewWindowBuilder},
    App, AppHandle, Emitter, Listener, RunEvent, WebviewUrl,
};

#[derive(Clone, Serialize)]
struct Reply {
    data: String,
}

pub type SetupHook = Box<dyn FnOnce(&mut App) -> Result<(), Box<dyn std::error::Error>> + Send>;
pub type OnEvent = Box<dyn FnMut(&AppHandle, RunEvent)>;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[allow(unused_mut)]
    let mut builder = tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::default()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .setup(move |app| {
            #[cfg(desktop)]
            {
                tray::create_tray(app.handle())?;
                app.handle().plugin(tauri_plugin_cli::init())?;
                app.handle()
                    .plugin(tauri_plugin_global_shortcut::Builder::new().build())?;
                app.handle()
                    .plugin(tauri_plugin_window_state::Builder::new().build())?;
                app.handle()
                    .plugin(tauri_plugin_updater::Builder::new().build())?;
            }
            #[cfg(mobile)]
            {
                app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
                app.handle().plugin(tauri_plugin_nfc::init())?;
                app.handle().plugin(tauri_plugin_biometric::init())?;
                app.handle().plugin(tauri_plugin_geolocation::init())?;
                app.handle().plugin(tauri_plugin_haptics::init())?;
            }

            let mut webview_window_builder =
                WebviewWindowBuilder::new(app, "main", WebviewUrl::default());
            #[cfg(desktop)]
            {
                webview_window_builder = webview_window_builder
                    .user_agent(&format!("Tauri API - {}", std::env::consts::OS))
                    .title("Tauri API Validation")
                    .inner_size(1000., 800.)
                    .min_inner_size(600., 400.)
                    .visible(false);
            }

            #[cfg(target_os = "windows")]
            {
                webview_window_builder = webview_window_builder
                    .transparent(true)
                    .shadow(true)
                    .decorations(false);
            }

            #[cfg(target_os = "macos")]
            {
                webview_window_builder = webview_window_builder.transparent(true);
            }

            let webview = webview_window_builder.build().unwrap();

            #[cfg(debug_assertions)]
            webview.open_devtools();

            std::thread::spawn(|| {
                let server = match tiny_http::Server::http("localhost:3003") {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("{}", e);
                        std::process::exit(1);
                    }
                };
                loop {
                    if let Ok(mut request) = server.recv() {
                        let mut body = Vec::new();
                        let _ = request.as_reader().read_to_end(&mut body);
                        let response = tiny_http::Response::new(
                            tiny_http::StatusCode(200),
                            request.headers().to_vec(),
                            std::io::Cursor::new(body),
                            request.body_length(),
                            None,
                        );
                        let _ = request.respond(response);
                    }
                }
            });

            Ok(())
        })
        .on_page_load(|webview, payload| {
            if payload.event() == PageLoadEvent::Finished {
                let webview_ = webview.clone();
                webview.listen("js-event", move |event| {
                    println!("got js-event with message '{:?}'", event.payload());
                    let reply = Reply {
                        data: "something else".to_string(),
                    };

                    webview_
                        .emit("rust-event", Some(reply))
                        .expect("failed to emit");
                });
            }
        });

    #[cfg(target_os = "macos")]
    {
        builder = builder.menu(tauri::menu::Menu::default);
    }

    #[allow(unused_mut)]
    let mut app = builder
        .invoke_handler(tauri::generate_handler![
            cmd::log_operation,
            cmd::perform_request,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    #[cfg(target_os = "macos")]
    app.set_activation_policy(tauri::ActivationPolicy::Regular);

    app.run(move |_app_handle, _event| {
        #[cfg(desktop)]
        if let RunEvent::ExitRequested { code, api, .. } = &_event {
            if code.is_none() {
                // Keep the event loop running even if all windows are closed
                // This allow us to catch system tray events when there is no window
                api.prevent_exit();
            }
        }
    })
}
