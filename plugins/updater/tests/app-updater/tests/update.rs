// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code, unused_imports)]

use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};

use serde::Serialize;
use tauri::utils::config::{Updater, V1Compatible};

const UPDATER_PRIVATE_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5TlFOMFpXYzJFOUdjeHJEVXY4WE1TMUxGNDJVUjNrMmk1WlR3UVJVUWwva0FBQkFBQUFBQUFBQUFBQUlBQUFBQUpVK3ZkM3R3eWhyN3hiUXhQb2hvWFVzUW9FbEs3NlNWYjVkK1F2VGFRU1FEaGxuRUtlell5U0gxYS9DbVRrS0YyZVJGblhjeXJibmpZeGJjS0ZKSUYwYndYc2FCNXpHalM3MHcrODMwN3kwUG9SOWpFNVhCSUd6L0E4TGRUT096TEtLR1JwT1JEVFU9Cg==";
const UPDATED_EXIT_CODE: i32 = 0;
const UP_TO_DATE_EXIT_CODE: i32 = 2;

#[derive(Serialize, Clone)]
struct Config {
    version: &'static str,
    bundle: BundleConfig,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BundleConfig {
    create_updater_artifacts: Updater,
}

#[derive(Serialize)]
struct Update {
    version: &'static str,
    date: String,
    signature: String,
    url: &'static str,
}

fn setup_test() -> (PathBuf, PathBuf, Config, Config) {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    let target_dir = std::env::var("CARGO_TARGET_DIR")
        .or_else(|_| std::env::var("CARGO_BUILD_TARGET_DIR"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../../../../target"));

    let base_config = Config {
        version: "0.1.0",
        bundle: BundleConfig {
            create_updater_artifacts: Updater::Bool(true),
        },
    };

    let config = Config {
        version: "1.0.0",
        bundle: BundleConfig {
            create_updater_artifacts: Updater::Bool(true),
        },
    };

    (manifest_dir, target_dir, base_config, config)
}

fn build_app(cwd: &Path, config: &Config, bundle_updater: bool, target: &str) {
    let mut command = Command::new("cargo");
    command
        .args(["tauri", "build", "--debug"])
        .arg("--config")
        .arg(serde_json::to_string(config).unwrap())
        .env("TAURI_SIGNING_PRIVATE_KEY", UPDATER_PRIVATE_KEY)
        .env("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "")
        .current_dir(cwd);

    if bundle_updater {
        command.args(["--bundles", target, "updater"]);
    } else {
        command.arg("--no-bundle");
    }

    let status = command
        .status()
        .expect("failed to run Tauri CLI to bundle app");

    if !status.success() {
        panic!("failed to bundle app {:?}", status.code());
    }
}

fn start_server(update_bundle: PathBuf, signature: PathBuf) -> Arc<tiny_http::Server> {
    let server = tiny_http::Server::http("localhost:3007").expect("failed to start updater server");
    let server = Arc::new(server);
    let server_ = server.clone();
    std::thread::spawn(move || {
        for request in server_.incoming_requests() {
            match request.url() {
                "/" => {
                    let signature =
                        std::fs::read_to_string(&signature).expect("failed to read signature");

                    let now = time::OffsetDateTime::now_utc()
                        .format(&time::format_description::well_known::Rfc3339)
                        .unwrap();

                    let body = serde_json::to_vec(&Update {
                        version: "1.0.0",
                        date: now,
                        signature,
                        url: "http://localhost:3007/download",
                    })
                    .unwrap();

                    let len = body.len();

                    let response = tiny_http::Response::new(
                        tiny_http::StatusCode(200),
                        Vec::new(),
                        std::io::Cursor::new(body),
                        Some(len),
                        None,
                    );

                    let _ = request.respond(response);
                }
                "/download" => {
                    let file = File::open(&update_bundle).unwrap_or_else(|_| {
                        panic!("failed to open updater bundle {}", update_bundle.display())
                    });

                    let _ = request.respond(tiny_http::Response::from_file(file));
                }
                _ => (),
            }
        }
    });

    server
}

fn test_update(app: &Path, update_bundle: PathBuf, signature: PathBuf, target: &str) {
    // start the updater server
    let server = start_server(update_bundle, signature);

    // run app
    let mut app_cmd = Command::new(app);
    #[cfg(target_os = "linux")]
    if std::env::var("CI").map(|v| v == "true").unwrap_or_default() {
        app_cmd = Command::new("xvfb-run");
        app_cmd.arg("--auto-servernum").arg(app);
    };
    app_cmd.env("TARGET", target);
    let output = app_cmd.output().expect("failed to run app");

    // check if updated, or failed during update
    let code = output.status.code().unwrap_or(-1);
    if code != UPDATED_EXIT_CODE {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!("app failed while updating, expected exit code {UPDATED_EXIT_CODE}, got {code}\n{stderr}");
    }

    // wait for the update to finish
    std::thread::sleep(std::time::Duration::from_secs(20));

    // run again
    let status = app_cmd.status().expect("failed to run new app");

    //  check if new version is up to date
    let code = status.code().unwrap_or(-1);
    if code != UP_TO_DATE_EXIT_CODE {
        let stderr = String::from_utf8_lossy(&output.stderr);
        panic!(
            "app failed to update, expected exit code {UP_TO_DATE_EXIT_CODE}, got {code}\n{stderr}"
        );
    }

    // shutdown the server
    server.unblock();
}

#[cfg(windows)]
fn nsis() {
    let (manifest_dir, target_dir, base_config, config) = setup_test();

    // build update bundles
    build_app(&manifest_dir, &config, true, "nsis");

    // bundle base app
    build_app(&manifest_dir, &base_config, false, "nsis");

    let app = target_dir.join("debug/app-updater.exe");

    // test nsis installer updates
    let update_bundle = target_dir.join(format!(
        "debug/bundle/nsis/app-updater_{}_x64-setup.exe",
        config.version,
    ));
    let signature = update_bundle.with_extension("exe.sig");
    test_update(&app, update_bundle, signature, "nsis");

    // cleanup the installed application
    if !std::env::var("CI").map(|v| v == "true").unwrap_or_default() {
        let _ = Command::new(target_dir.join("debug/uninstall.exe"))
            .arg("/S")
            .status()
            .expect("failed to run nsis uninstaller");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

#[cfg(windows)]
fn msi() {
    let (manifest_dir, target_dir, base_config, config) = setup_test();

    // build update bundles
    build_app(&manifest_dir, &config, true, "msi");

    // bundle base app
    build_app(&manifest_dir, &base_config, false, "msi");

    let app = target_dir.join("debug/app-updater.exe");

    // test msi installer updates
    let update_bundle = target_dir.join(format!(
        "debug/bundle/msi/app-updater_{}_x64_en-US.msi",
        config.version,
    ));
    let signature = update_bundle.with_extension("msi.sig");
    test_update(&app, update_bundle, signature, "msi");

    // cleanup the installed application
    if !std::env::var("CI").map(|v| v == "true").unwrap_or_default() {
        let uninstall = target_dir.join("debug/Uninstall app-updater.lnk");
        let _ = Command::new("cmd")
            .arg("/c")
            .arg(&uninstall)
            .arg("/quiet")
            .status()
            .expect("failed to run msi uninstaller");
        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}

#[cfg(target_os = "linux")]
fn appimage() {
    let (manifest_dir, target_dir, base_config, config) = setup_test();

    // build update bundles
    build_app(&manifest_dir, &config, true, "appimage");

    let update_bundle = target_dir.join(format!(
        "debug/bundle/appimage/app-updater_{}_amd64.AppImage",
        config.version,
    ));
    let signature = update_bundle.with_extension("AppImage.sig");

    // backup update bundles files because next build will override them
    let appimage_backup = target_dir.join("debug/bundle/test-appimage.AppImage");
    let signature_backup = target_dir.join("debug/bundle/test-appimage.AppImage.sig");
    std::fs::rename(&update_bundle, &appimage_backup);
    std::fs::rename(&signature, &signature_backup);

    // bundle base app
    build_app(&manifest_dir, &base_config, true, "appimage");

    // restore backup
    std::fs::rename(&appimage_backup, &update_bundle);
    std::fs::rename(&signature_backup, &signature);

    let app = target_dir.join(format!(
        "debug/bundle/appimage/app-updater_{}_amd64.AppImage",
        base_config.version
    ));

    // test appimage updates
    test_update(&app, update_bundle, signature, "appimage");
}

#[cfg(target_os = "macos")]
fn app() {
    let (manifest_dir, target_dir, base_config, config) = setup_test();
}

#[test]
#[ignore]
fn it_updates() {
    #[cfg(windows)]
    nsis();
    // MSI test should be the last one
    #[cfg(windows)]
    msi();
    #[cfg(target_os = "linux")]
    appimage();
    #[cfg(target_os = "macos")]
    app();
}
