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
            tauri::async_runtime::spawn(async move {
                let mut builder = handle.updater_builder();
                if std::env::var("TARGET").unwrap_or_default() == "nsis" {
                    // /D sets the default installation directory ($INSTDIR),
                    // overriding InstallDir and InstallDirRegKey.
                    // It must be the last parameter used in the command line and must not contain any quotes, even if the path contains spaces.
                    // Only absolute paths are supported.
                    // NOTE: we only need this because this is an integration test and we don't want to install the app in the programs folder
                    builder = builder.installer_args(vec![format!(
                        "/D={}",
                        tauri::utils::platform::current_exe()
                            .unwrap()
                            .parent()
                            .unwrap()
                            .display()
                    )]);
                }
                let updater = builder.build().unwrap();

                match updater.check().await {
                    Ok(Some(update)) => {
                        if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                            println!("{e}");
                            std::process::exit(1);
                        }
                        std::process::exit(0);
                    }
                    Ok(None) => {
                        std::process::exit(0);
                    }
                    Err(e) => {
                        println!("{e}");
                        std::process::exit(1);
                    }
                }
            });
            Ok(())
        })
        .run(context)
        .expect("error while running tauri application");
}
