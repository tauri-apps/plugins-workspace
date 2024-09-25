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

fn npm_command() -> Command {
    #[cfg(target_os = "windows")]
    let cmd = {
        let mut cmd = Command::new("cmd");
        cmd.arg("/c").arg("npm");
        cmd
    };
    #[cfg(not(target_os = "windows"))]
    let cmd = Command::new("npm");
    cmd
}

mod v1 {
    use super::{npm_command, BundleTarget, UPDATER_PRIVATE_KEY};
    use serde::Serialize;
    use std::{
        path::{Path, PathBuf},
        process::Command,
    };

    #[derive(Serialize)]
    pub struct PackageConfig {
        pub version: &'static str,
    }

    #[derive(Serialize)]
    pub struct Config {
        pub package: PackageConfig,
    }

    pub fn build_app(cwd: &Path, config: &Config, bundle_updater: bool, target: BundleTarget) {
        let mut command = npm_command();
        command
            .args(["run", "tauri", "--", "build", "--debug", "--verbose"])
            .arg("--config")
            .arg(serde_json::to_string(config).unwrap())
            .env("TAURI_PRIVATE_KEY", UPDATER_PRIVATE_KEY)
            .env("TAURI_KEY_PASSWORD", "")
            .current_dir(cwd);

        #[cfg(target_os = "linux")]
        command.args(["--bundles", target.name()]);
        #[cfg(target_os = "macos")]
        command.args(["--bundles", target.name()]);

        if bundle_updater {
            #[cfg(windows)]
            command.args(["--bundles", "msi", "nsis"]);

            command.args(["--bundles", "updater"]);
        } else {
            #[cfg(windows)]
            command.args(["--bundles", target.name()]);
        }

        let status = command
            .status()
            .expect("failed to run Tauri CLI to bundle v1 app");

        if !status.success() {
            panic!("failed to bundle v1 app {:?}", status.code());
        }
    }

    #[cfg(target_os = "linux")]
    pub fn bundle_paths(root_dir: &Path, version: &str) -> Vec<(BundleTarget, PathBuf)> {
        vec![(
            BundleTarget::AppImage,
            root_dir.join(format!(
                "target/debug/bundle/appimage/app-updater-v1_{version}_amd64.AppImage",
            )),
        )]
    }

    #[cfg(target_os = "macos")]
    pub fn bundle_paths(root_dir: &Path, _version: &str) -> Vec<(BundleTarget, PathBuf)> {
        vec![(
            BundleTarget::App,
            root_dir.join("target/debug/bundle/macos/app-updater-v1.app"),
        )]
    }

    #[cfg(target_os = "ios")]
    pub fn bundle_paths(root_dir: &Path, _version: &str) -> Vec<(BundleTarget, PathBuf)> {
        vec![(
            BundleTarget::App,
            root_dir.join("target/debug/bundle/ios/app-updater-v1.ipa"),
        )]
    }

    #[cfg(target_os = "android")]
    pub fn bundle_path(root_dir: &Path, _version: &str) -> PathBuf {
        root_dir.join("target/debug/bundle/android/app-updater-v1.apk")
    }

    #[cfg(windows)]
    pub fn bundle_paths(root_dir: &Path, version: &str) -> Vec<(BundleTarget, PathBuf)> {
        vec![
            (
                BundleTarget::Nsis,
                root_dir.join(format!(
                    "target/debug/bundle/nsis/app-updater-v1_{version}_x64-setup.exe"
                )),
            ),
            (
                BundleTarget::Msi,
                root_dir.join(format!(
                    "target/debug/bundle/msi/app-updater-v1_{version}_x64_en-US.msi"
                )),
            ),
        ]
    }
}

mod v2 {

    use super::{BundleTarget, UPDATER_PRIVATE_KEY};
    use serde::Serialize;
    use std::{
        path::{Path, PathBuf},
        process::Command,
    };
    use tauri::utils::config::Updater;

    #[derive(Serialize)]
    pub struct Config {
        pub version: &'static str,
        pub bundle: BundleConfig,
    }

    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    pub struct BundleConfig {
        pub create_updater_artifacts: Updater,
    }

    pub fn build_app(cwd: &Path, config: &Config, bundle_updater: bool, target: BundleTarget) {
        let mut command = Command::new("cargo");
        command
            .args(["tauri", "build", "--debug", "--verbose"])
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

            command.args(["--bundles", "updater"]);
        } else {
            #[cfg(windows)]
            command.args(["--bundles", target.name()]);
        }

        let status = command
            .status()
            .expect("failed to run Tauri CLI to bundle v2 app");

        if !status.success() {
            panic!("failed to bundle v2 app {:?}", status.code());
        }
    }

    #[cfg(target_os = "linux")]
    pub fn bundle_paths(root_dir: &Path, version: &str) -> Vec<(BundleTarget, PathBuf)> {
        vec![(
            BundleTarget::AppImage,
            root_dir.join(format!(
                "target/debug/bundle/appimage/app-updater-v2_{version}_amd64.AppImage",
            )),
        )]
    }

    #[cfg(target_os = "macos")]
    pub fn bundle_paths(root_dir: &Path, _version: &str) -> Vec<(BundleTarget, PathBuf)> {
        vec![(
            BundleTarget::App,
            root_dir.join("target/debug/bundle/macos/app-updater-v2.app"),
        )]
    }

    #[cfg(target_os = "ios")]
    pub fn bundle_paths(root_dir: &Path, _version: &str) -> Vec<(BundleTarget, PathBuf)> {
        vec![(
            BundleTarget::App,
            root_dir.join("target/debug/bundle/ios/app-updater-v2.ipa"),
        )]
    }

    #[cfg(target_os = "android")]
    pub fn bundle_path(root_dir: &Path, _version: &str) -> PathBuf {
        root_dir.join("target/debug/bundle/android/app-updater-v2.apk")
    }

    #[cfg(windows)]
    pub fn bundle_paths(root_dir: &Path, version: &str) -> Vec<(BundleTarget, PathBuf)> {
        vec![
            (
                BundleTarget::Nsis,
                root_dir.join(format!(
                    "target/debug/bundle/nsis/app-updater-v2_{version}_x64-setup.exe"
                )),
            ),
            (
                BundleTarget::Msi,
                root_dir.join(format!(
                    "target/debug/bundle/msi/app-updater-v2_{version}_x64_en-US.msi"
                )),
            ),
        ]
    }
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

#[derive(Copy, Clone)]
enum BundleTarget {
    AppImage,

    App,

    Msi,
    Nsis,
}

impl BundleTarget {
    fn name(self) -> &'static str {
        match self {
            Self::AppImage => "appimage",
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

#[test]
#[ignore]
fn update_app() {
    let target =
        tauri_plugin_updater::target().expect("running updater test in an unsupported platform");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.join("../../../..");
    let v1_root_dir = manifest_dir.join("v1-app");
    let v2_root_dir = manifest_dir.join("v2-app");

    let status = npm_command()
        .arg("install")
        .current_dir(&v1_root_dir)
        .status()
        .expect("failed to run npm install");
    if !status.success() {
        panic!("failed to run npm install");
    }

    let v2_config = v2::Config {
        version: "1.0.0",
        bundle: v2::BundleConfig {
            create_updater_artifacts: Updater::String(V1Compatible::V1Compatible),
        },
    };

    // bundle app update (v2)
    v2::build_app(&v2_root_dir, &v2_config, true, Default::default());

    let updater_zip_ext = if cfg!(windows) { "zip" } else { "tar.gz" };

    for (bundle_target, out_bundle_path) in v2::bundle_paths(&root_dir, "1.0.0") {
        let bundle_updater_ext = out_bundle_path
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .replace("exe", "nsis");
        let updater_extension = format!("{bundle_updater_ext}.{updater_zip_ext}");
        let signature_extension = format!("{updater_extension}.sig");
        let signature_path = out_bundle_path.with_extension(signature_extension);
        let signature = std::fs::read_to_string(&signature_path).unwrap_or_else(|_| {
            panic!("failed to read signature file {}", signature_path.display())
        });
        let out_updater_path = out_bundle_path.with_extension(updater_extension);
        let updater_path = root_dir.join(format!(
            "target/debug/{}",
            out_updater_path.file_name().unwrap().to_str().unwrap()
        ));
        std::fs::rename(&out_updater_path, &updater_path).expect("failed to rename bundle");

        let target = target.clone();

        // start the updater server
        let server = Arc::new(
            tiny_http::Server::http("localhost:3007").expect("failed to start updater server"),
        );

        let server_ = server.clone();
        std::thread::spawn(move || {
            for request in server_.incoming_requests() {
                match request.url() {
                    "/" => {
                        let mut platforms = HashMap::new();

                        platforms.insert(
                            target.clone(),
                            PlatformUpdate {
                                signature: signature.clone(),
                                url: "http://localhost:3007/download",
                                with_elevated_task: false,
                            },
                        );
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
                                panic!("failed to open updater bundle {}", updater_path.display())
                            }),
                        ));
                    }
                    _ => (),
                }
            }
        });

        let v1_config = v1::Config {
            package: v1::PackageConfig { version: "0.1.0" },
        };

        // bundle initial app version (tauri v1)
        v1::build_app(&v1_root_dir, &v1_config, false, bundle_target);

        let status_checks = if matches!(bundle_target, BundleTarget::Msi) {
            // for msi we can't really check if the app was updated, because we can't change the install path
            vec![(UPDATED_EXIT_CODE, 1)]
        } else {
            vec![(UPDATED_EXIT_CODE, 1), (UP_TO_DATE_EXIT_CODE, 2)]
        };

        for (expected_exit_code, expected_tauri_version) in status_checks {
            let (expected_app_version, bundle_paths_fn, app_name_suffix) =
                match expected_tauri_version {
                    1 => (
                        v1_config.package.version,
                        Box::new(|| v1::bundle_paths(&v1_root_dir, v1_config.package.version))
                            as Box<dyn Fn() -> Vec<(BundleTarget, PathBuf)>>,
                        "-v1",
                    ),
                    2 => (
                        v2_config.version,
                        Box::new(|| v2::bundle_paths(&root_dir, v2_config.version))
                            as Box<dyn Fn() -> Vec<(BundleTarget, PathBuf)>>,
                        "-v2",
                    ),
                    _ => panic!("unknown tauri version"),
                };
            let mut binary_cmd = if cfg!(windows) {
                let app_root_dir = match expected_tauri_version {
                    1 => &v1_root_dir,
                    2 => &root_dir,
                    _ => panic!("unknown tauri version"),
                };
                Command::new(
                    app_root_dir.join(format!("target/debug/app-updater{app_name_suffix}.exe")),
                )
            } else if cfg!(target_os = "macos") {
                Command::new(
                    bundle_paths_fn()
                        .first()
                        .unwrap()
                        .1
                        .join(format!("Contents/MacOS/app-updater{app_name_suffix}")),
                )
            } else if std::env::var("CI").map(|v| v == "true").unwrap_or_default() {
                let mut c = Command::new("xvfb-run");
                c.arg("--auto-servernum")
                    .arg(&bundle_paths_fn().first().unwrap().1);
                c
            } else {
                Command::new(&bundle_paths_fn().first().unwrap().1)
            };

            binary_cmd.env("TARGET", bundle_target.name());

            let output = binary_cmd.output().expect("failed to run app");
            let stdout = String::from_utf8_lossy(&output.stdout);

            println!("{stdout}");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));

            let code = output.status.code().unwrap_or(-1);

            if code != expected_exit_code {
                panic!("failed to run app, expected exit code {expected_exit_code}, got {code}");
            }
            if !stdout.contains(&format!("version={expected_app_version}")) {
                panic!("app version does not match {expected_app_version}");
            }
            #[cfg(windows)]
            if code == UPDATED_EXIT_CODE {
                // wait for the update to finish
                std::thread::sleep(std::time::Duration::from_secs(5));
            }
        }

        server.unblock();
    }
}
