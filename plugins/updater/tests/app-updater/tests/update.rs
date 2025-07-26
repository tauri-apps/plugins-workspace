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
const ERROR_EXIT_CODE: i32 = 1;
const UP_TO_DATE_EXIT_CODE: i32 = 2;

#[derive(Serialize)]
struct Config {
    version: &'static str,
    bundle: BundleConfig,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleConfig {
    create_updater_artifacts: Updater,
}

#[derive(Serialize)]
struct PlatformUpdate {
    signature: String,
    url: &'static str,
    with_elevated_task: bool,
}

#[derive(Serialize)]
struct Update {
    version: &'static str,
    date: String,
    platforms: HashMap<String, PlatformUpdate>,
}

fn build_app(cwd: &Path, config: &Config, bundle_updater: bool, target: BundleTarget) {
    let mut command = Command::new("cargo");
    command
        .args(["tauri", "build", "--verbose"])
        .arg("--config")
        .arg(serde_json::to_string(config).unwrap())
        .env("TAURI_SIGNING_PRIVATE_KEY", UPDATER_PRIVATE_KEY)
        .env("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "")
        .current_dir(cwd);

    #[cfg(target_os = "linux")]
    command.args(["--bundles", target.name()]);
    #[cfg(target_os = "macos")]
    command.args(["--bundles", target.name()]);

    if bundle_updater {
        #[cfg(windows)]
        command.args(["--bundles", "msi", "nsis"]);
    } else {
        #[cfg(windows)]
        command.args(["--bundles", target.name()]);
    }

    let status = command
        .status()
        .expect("failed to run Tauri CLI to bundle app");

    if !status.success() {
        panic!("failed to bundle app {:?}", status.code());
    }
}

#[derive(Copy, Clone)]
enum BundleTarget {
    AppImage,
    Deb,
    Rpm,

    App,

    Msi,
    Nsis,
}

impl BundleTarget {
    fn name(self) -> &'static str {
        match self {
            Self::AppImage => "appimage",
            Self::Deb => "deb",
            Self::Rpm => "rpm",
            Self::App => "app",
            Self::Msi => "msi",
            Self::Nsis => "nsis",
        }
    }
}

impl Default for BundleTarget {
    fn default() -> Self {
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        return Self::App;
        #[cfg(target_os = "linux")]
        return Self::AppImage;
        #[cfg(windows)]
        return Self::Nsis;
    }
}

fn target_to_platforms(
    update_platform: Option<String>,
    signature: String,
) -> HashMap<String, PlatformUpdate> {
    let mut platforms = HashMap::new();
    if let Some(platform) = update_platform {
        platforms.insert(
            platform,
            PlatformUpdate {
                signature,
                url: "http://localhost:3007/download",
                with_elevated_task: false,
            },
        );
    }

    platforms
}

#[cfg(target_os = "linux")]
fn test_cases(
    root_dir: &Path,
    version: &str,
    target: String,
) -> Vec<(BundleTarget, PathBuf, Option<String>, Vec<i32>)> {
    vec![
        // update using fallback
        (
            BundleTarget::AppImage,
            root_dir.join(format!(
                "target/release/bundle/appimage/app-updater_{version}_amd64.AppImage"
            )),
            Some(target.clone()),
            vec![UPDATED_EXIT_CODE, UP_TO_DATE_EXIT_CODE],
        ),
        // update using full name
        (
            BundleTarget::AppImage,
            root_dir.join(format!(
                "target/release/bundle/appimage/app-updater_{version}_amd64.AppImage"
            )),
            Some(format!("{target}-{}", BundleTarget::AppImage.name())),
            vec![UPDATED_EXIT_CODE, UP_TO_DATE_EXIT_CODE],
        ),
        // no update
        (
            BundleTarget::AppImage,
            root_dir.join(format!(
                "target/release/bundle/appimage/app-updater_{version}_amd64.AppImage"
            )),
            None,
            vec![ERROR_EXIT_CODE],
        ),
    ]
}

#[cfg(target_os = "macos")]
fn test_cases(
    root_dir: &Path,
    _version: &str,
    target: String,
) -> Vec<(BundleTarget, PathBuf, Option<String>, Vec<i32>)> {
    vec![
        (
            BundleTarget::App,
            root_dir.join("target/release/bundle/macos/app-updater.app"),
            Some(target.clone()),
            vec![UPDATED_EXIT_CODE, UP_TO_DATE_EXIT_CODE],
        ),
        // update with installer
        (
            BundleTarget::App,
            root_dir.join("target/release/bundle/macos/app-updater.app"),
            Some(format!("{target}-{}", BundleTarget::App.name())),
            vec![UPDATED_EXIT_CODE, UP_TO_DATE_EXIT_CODE],
        ),
        // no update
        (
            BundleTarget::App,
            root_dir.join("target/release/bundle/macos/app-updater.app"),
            None,
            vec![ERROR_EXIT_CODE],
        ),
    ]
}

#[cfg(target_os = "ios")]
fn bundle_paths(
    root_dir: &Path,
    _version: &str,
    v1compatible: bool,
) -> Vec<(BundleTarget, PathBuf)> {
    vec![(
        BundleTarget::App,
        root_dir.join("target/release/bundle/ios/app-updater.ipa"),
    )]
}

#[cfg(target_os = "android")]
fn bundle_path(root_dir: &Path, _version: &str, v1compatible: bool) -> PathBuf {
    root_dir.join("target/release/bundle/android/app-updater.apk")
}

#[cfg(windows)]
fn test_cases(
    root_dir: &Path,
    version: &str,
    target: String,
) -> Vec<(BundleTarget, PathBuf, Option<String>, Vec<i32>)> {
    vec![
        (
            BundleTarget::Nsis,
            root_dir.join(format!(
                "target/release/bundle/nsis/app-updater_{version}_x64-setup.exe"
            )),
            Some(target.clone()),
            vec![UPDATED_EXIT_CODE],
        ),
        (
            BundleTarget::Nsis,
            root_dir.join(format!(
                "target/release/bundle/nsis/app-updater_{version}_x64-setup.exe"
            )),
            Some(format!("{target}-{}", BundleTarget::Nsis.name())),
            vec![UPDATED_EXIT_CODE],
        ),
        (
            BundleTarget::Nsis,
            root_dir.join(format!(
                "target/release/bundle/nsis/app-updater_{version}_x64-setup.exe"
            )),
            None,
            vec![ERROR_EXIT_CODE],
        ),
        (
            BundleTarget::Msi,
            root_dir.join(format!(
                "target/release/bundle/msi/app-updater_{version}_x64_en-US.msi"
            )),
            Some(target.clone()),
            vec![UPDATED_EXIT_CODE],
        ),
        (
            BundleTarget::Msi,
            root_dir.join(format!(
                "target/release/bundle/msi/app-updater_{version}_x64_en-US.msi"
            )),
            Some(format!("{target}-{}", BundleTarget::Msi.name())),
            vec![UPDATED_EXIT_CODE],
        ),
        (
            BundleTarget::Msi,
            root_dir.join(format!(
                "target/release/bundle/msi/app-updater_{version}_x64_en-US.msi"
            )),
            None,
            vec![ERROR_EXIT_CODE],
        ),
    ]
}

#[test]
fn update_app() {
    let target =
        tauri_plugin_updater::target().expect("running updater test in an unsupported platform");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.join("../../../..");

    for mut config in [
        Config {
            version: "1.0.0",
            bundle: BundleConfig {
                create_updater_artifacts: Updater::Bool(true),
            },
        },
        Config {
            version: "1.0.0",
            bundle: BundleConfig {
                create_updater_artifacts: Updater::String(V1Compatible::V1Compatible),
            },
        },
    ] {
        let v1_compatible = matches!(
            config.bundle.create_updater_artifacts,
            Updater::String(V1Compatible::V1Compatible)
        );

        let updater_zip_ext = if v1_compatible {
            if cfg!(windows) {
                Some("zip")
            } else {
                Some("tar.gz")
            }
        } else if cfg!(target_os = "macos") {
            Some("tar.gz")
        } else {
            None
        };

        for (bundle_target, out_bundle_path, update_platform, status_checks) in
            test_cases(&root_dir, "1.0.0", target.clone())
        {
            // bundle app update
            config.version = "1.0.0";
            build_app(&manifest_dir, &config, true, BundleTarget::default());

            let bundle_updater_ext = if v1_compatible {
                out_bundle_path
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .replace("exe", "nsis")
            } else {
                out_bundle_path
                    .extension()
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string()
            };
            let updater_extension = if let Some(updater_zip_ext) = updater_zip_ext {
                format!("{bundle_updater_ext}.{updater_zip_ext}")
            } else {
                bundle_updater_ext
            };
            let signature_extension = format!("{updater_extension}.sig");
            let signature_path = out_bundle_path.with_extension(signature_extension);
            let signature = std::fs::read_to_string(&signature_path).unwrap_or_else(|_| {
                panic!("failed to read signature file {}", signature_path.display())
            });
            let out_updater_path = out_bundle_path.with_extension(updater_extension);
            let updater_path = root_dir.join(format!(
                "target/release/{}",
                out_updater_path.file_name().unwrap().to_str().unwrap()
            ));
            std::fs::rename(&out_updater_path, &updater_path).expect("failed to rename bundle");

            // start the updater server
            let server = Arc::new(
                tiny_http::Server::http("localhost:3007").expect("failed to start updater server"),
            );

            let server_ = server.clone();
            std::thread::spawn(move || {
                for request in server_.incoming_requests() {
                    match request.url() {
                        "/" => {
                            let platforms =
                                target_to_platforms(update_platform.clone(), signature.clone());

                            let body = serde_json::to_vec(&Update {
                                version: "1.0.0",
                                date: time::OffsetDateTime::now_utc()
                                    .format(&time::format_description::well_known::Rfc3339)
                                    .unwrap(),
                                platforms,
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
                            let _ = request.respond(tiny_http::Response::from_file(
                                File::open(&updater_path).unwrap_or_else(|_| {
                                    panic!(
                                        "failed to open updater bundle {}",
                                        updater_path.display()
                                    )
                                }),
                            ));
                        }
                        _ => (),
                    }
                }
            });

            config.version = "0.1.0";

            // bundle initial app version
            build_app(&manifest_dir, &config, false, bundle_target);

            for expected_exit_code in status_checks {
                let mut binary_cmd = if cfg!(windows) {
                    Command::new(root_dir.join("target/release/app-updater.exe"))
                } else if cfg!(target_os = "macos") {
                    Command::new(
                        test_cases(&root_dir, "0.1.0", target.clone())
                            .first()
                            .unwrap()
                            .1
                            .join("Contents/MacOS/app-updater"),
                    )
                } else if std::env::var("CI").map(|v| v == "true").unwrap_or_default() {
                    let mut c = Command::new("xvfb-run");
                    c.arg("--auto-servernum").arg(
                        &test_cases(&root_dir, "0.1.0", target.clone())
                            .first()
                            .unwrap()
                            .1,
                    );
                    c
                } else {
                    Command::new(
                        &test_cases(&root_dir, "0.1.0", target.clone())
                            .first()
                            .unwrap()
                            .1,
                    )
                };

                binary_cmd.env("TARGET", bundle_target.name());

                let status = binary_cmd.status().expect("failed to run app");
                let code = status.code().unwrap_or(-1);

                if code != expected_exit_code {
                    panic!(
                        "failed to run app bundled as {}, expected exit code {expected_exit_code}, got {code}", bundle_target.name()
                    );
                }
                #[cfg(windows)]
                if code == UPDATED_EXIT_CODE {
                    // wait for the update to finish
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }
            }

            // graceful shutdown
            server.unblock();
        }
    }
}
