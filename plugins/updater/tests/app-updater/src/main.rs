// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri_plugin_updater::UpdaterExt;

fn main() {
    #[allow(unused_mut)]
    let mut context = tauri::generate_context!();

    tauri::Builder::default()
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            let handle = app.handle().clone();
            eprintln!("app version: {}", app.package_info().version);

            tauri::async_runtime::spawn(async move {
                #[allow(unused_mut)]
                let mut builder = handle.updater_builder();

                // Overriding installation directory for integration tests on Windows
                #[cfg(windows)]
                {
                    let target = std::env::var("TARGET").unwrap_or_default();
                    let exe = tauri::utils::platform::current_exe().unwrap();
                    let dir = dunce::simplified(exe.parent().unwrap()).display();
                    if target == "nsis" {
                        builder = builder.installer_arg(format!("/D=\"{dir}\"",));
                    } else if target == "msi" {
                        builder = builder.installer_arg(format!("INSTALLDIR=\"{dir}\""));
                    }
                }

                let updater = builder.build().unwrap();

                match updater.check().await {
                    Ok(Some(update)) => {
                        if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                            eprintln!("{e}");
                            std::process::exit(1);
                        }
                        std::process::exit(0);
                    }
                    Ok(None) => {
                        std::process::exit(2);
                    }
                    Err(e) => {
                        eprintln!("{e}");
                        std::process::exit(3);
                    }
                }
            });
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
