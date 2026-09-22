// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(dead_code, unused_imports)]

use std::{
    collections::HashMap,
    fs::File,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Arc, Mutex},
};

use serde::Serialize;
use tauri::utils::config::{Updater, V1Compatible};

/// Every test here drives `cargo tauri build` against the same crate and target directory, and
/// binds a fixed port, so they cannot overlap.
static BUILD_LOCK: Mutex<()> = Mutex::new(());

const UPDATER_PRIVATE_KEY: &str = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5TlFOMFpXYzJFOUdjeHJEVXY4WE1TMUxGNDJVUjNrMmk1WlR3UVJVUWwva0FBQkFBQUFBQUFBQUFBQUlBQUFBQUpVK3ZkM3R3eWhyN3hiUXhQb2hvWFVzUW9FbEs3NlNWYjVkK1F2VGFRU1FEaGxuRUtlell5U0gxYS9DbVRrS0YyZVJGblhjeXJibmpZeGJjS0ZKSUYwYndYc2FCNXpHalM3MHcrODMwN3kwUG9SOWpFNVhCSUd6L0E4TGRUT096TEtLR1JwT1JEVFU9Cg==";
const UPDATED_EXIT_CODE: i32 = 0;
const ERROR_EXIT_CODE: i32 = 1;
const UP_TO_DATE_EXIT_CODE: i32 = 2;

#[derive(Serialize)]
struct Config {
    version: &'static str,
    bundle: BundleConfig,
    /// Merged into `plugins` of the app's `tauri.conf.json`. Only set by the signed version
    /// tests, which need to flip `requireSignedVersion` and point at their own server.
    #[serde(skip_serializing_if = "Option::is_none")]
    plugins: Option<serde_json::Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BundleConfig {
    create_updater_artifacts: Updater,
}

#[derive(Serialize)]
struct PlatformUpdate {
    signature: String,
    url: String,
    with_elevated_task: bool,
}

#[derive(Serialize)]
struct Update {
    version: String,
    date: String,
    platforms: HashMap<String, PlatformUpdate>,
}

fn build_app(cwd: &Path, config: &Config, target: Option<BundleTarget>) {
    let mut command = Command::new("cargo");
    command
        .args(["tauri", "build", "--verbose"])
        .arg("--config")
        .arg(serde_json::to_string(config).unwrap())
        .env("TAURI_SIGNING_PRIVATE_KEY", UPDATER_PRIVATE_KEY)
        .env("TAURI_SIGNING_PRIVATE_KEY_PASSWORD", "")
        .current_dir(cwd);

    // linuxdeploy ships a `strip` that does not know the `.relr.dyn` sections recent distros emit,
    // and it aborts the whole bundle when a strip call fails. Nothing here benefits from a smaller
    // AppImage, so skip it.
    #[cfg(target_os = "linux")]
    command.env("NO_STRIP", "1");

    if let Some(target) = target {
        command.arg("--bundles").arg(target.name());
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

#[cfg(target_os = "linux")]
impl BundleTarget {
    /// Where `cargo tauri build --bundles <name>` leaves the bundle of the given app version.
    fn bundle_path(self, root_dir: &Path, version: &str) -> PathBuf {
        root_dir.join(match self {
            Self::AppImage => {
                format!("target/release/bundle/appimage/app-updater_{version}_amd64.AppImage")
            }
            Self::Deb => format!("target/release/bundle/deb/app-updater_{version}_amd64.deb"),
            Self::Rpm => format!("target/release/bundle/rpm/app-updater-{version}-1.x86_64.rpm"),
            _ => unreachable!("{} is not a Linux bundle", self.name()),
        })
    }

    /// The package manager the updater hands this bundle to, for the Linux packages.
    fn package_manager(self) -> Option<&'static str> {
        match self {
            Self::Deb => Some("dpkg"),
            Self::Rpm => Some("rpm"),
            _ => None,
        }
    }

    /// Whether this host can install the package without asking anything: the package manager must
    /// be present and know the app's runtime dependencies, and `sudo` must not need a password. The
    /// update itself escalates through `pkexec`, then a graphical password prompt, then `sudo`, and
    /// in a non-interactive run only the last one can succeed.
    fn installable(self) -> Result<(), String> {
        let Some(manager) = self.package_manager() else {
            return Ok(());
        };
        // the bundler declares webkit2gtk as a dependency of every app, so an install is checked
        // against it
        let dependency_query: &[&str] = match self {
            Self::Deb => &["-s", "libwebkit2gtk-4.1-0"],
            _ => &["-q", "--whatprovides", "libwebkit2gtk-4.1.so.0()(64bit)"],
        };
        if !command_succeeds(manager, dependency_query) {
            return Err(format!(
                "{manager} is not installed or its database does not have the webkit2gtk runtime"
            ));
        }
        if !command_succeeds("sudo", &["-n", "true"]) {
            return Err("sudo needs a password".into());
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
fn command_succeeds(program: &str, args: &[&str]) -> bool {
    Command::new(program)
        .args(args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

/// Installs the package system wide, which is where the updater will later install the update over
/// it, and returns the path of the installed executable.
#[cfg(target_os = "linux")]
fn install_package(bundle_target: BundleTarget, package: &Path) -> PathBuf {
    let manager = bundle_target
        .package_manager()
        .expect("not a Linux package");
    // whatever an earlier run left behind
    remove_package(bundle_target);
    let status = Command::new("sudo")
        .args(["-n", manager, "-i"])
        .arg(package)
        .status()
        .expect("failed to run sudo");
    assert!(status.success(), "failed to install {}", package.display());
    PathBuf::from("/usr/bin/app-updater")
}

#[cfg(target_os = "linux")]
fn remove_package(bundle_target: BundleTarget) {
    let (manager, remove) = match bundle_target {
        BundleTarget::Deb => ("dpkg", "-r"),
        BundleTarget::Rpm => ("rpm", "-e"),
        _ => return,
    };
    let _ = Command::new("sudo")
        .args(["-n", manager, remove, "app-updater"])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();
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

/// Copies a bundle out of the bundler output directory, returning the path to run.
///
/// Every bundler wipes its output before writing — the AppImage one removes `bundle/appimage`
/// entirely, the macOS one the `.app` — so the app built for the initial version only survives
/// until the next `build_app` call. On Linux and macOS the update is also installed over the very
/// bundle being run, which would otherwise leave a 1.0.0 app behind in the bundler output.
///
/// Windows drives the executable `cargo build` leaves in `target/release`, which no bundler
/// touches, so there is nothing to copy there.
///
/// A Linux package is not copied but installed: the updater hands the downloaded package to the
/// package manager, which installs it over the one the app runs from.
fn stage_app_under_test(root_dir: &Path, target: &str, bundle_target: BundleTarget) -> PathBuf {
    #[cfg(windows)]
    {
        let _ = (target, bundle_target);
        return root_dir.join("target/release/app-updater.exe");
    }

    #[cfg(target_os = "linux")]
    {
        if bundle_target.package_manager().is_some() {
            return install_package(bundle_target, &bundle_target.bundle_path(root_dir, "0.1.0"));
        }
    }

    #[cfg(not(windows))]
    {
        let _ = bundle_target;
        let bundle_path = test_cases(root_dir, "0.1.0", target.to_string())
            .first()
            .unwrap()
            .1
            .clone();

        let staging_dir = root_dir.join("target/release/app-under-test");
        let _ = std::fs::remove_dir_all(&staging_dir);
        std::fs::create_dir_all(&staging_dir).expect("failed to create the staging directory");

        let staged = staging_dir.join(bundle_path.file_name().unwrap());
        copy_recursively(&bundle_path, &staged).unwrap_or_else(|e| {
            panic!(
                "failed to copy {} to {}: {e}",
                bundle_path.display(),
                staged.display()
            )
        });

        return staged;
    }
}

/// Copies a file, or a directory such as a macOS `.app`, preserving permissions.
fn copy_recursively(from: &Path, to: &Path) -> std::io::Result<()> {
    if from.is_dir() {
        std::fs::create_dir_all(to)?;
        for entry in std::fs::read_dir(from)? {
            let entry = entry?;
            copy_recursively(&entry.path(), &to.join(entry.file_name()))?;
        }
        Ok(())
    } else {
        std::fs::copy(from, to).map(|_| ())
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
                url: "http://localhost:3007/download".into(),
                with_elevated_task: false,
            },
        );
    }

    platforms
}

/// A bundle to build, the update platform key the server announces it under (none for a manifest
/// without an update) and the exit codes expected from running the app once per entry.
type TestCase = (BundleTarget, PathBuf, Option<String>, Vec<i32>);

#[cfg(target_os = "linux")]
fn test_cases(root_dir: &Path, version: &str, target: String) -> Vec<TestCase> {
    let appimage = BundleTarget::AppImage.bundle_path(root_dir, version);
    vec![
        // update using fallback
        (
            BundleTarget::AppImage,
            appimage.clone(),
            Some(target.clone()),
            vec![UPDATED_EXIT_CODE, UP_TO_DATE_EXIT_CODE],
        ),
        // update using full name
        (
            BundleTarget::AppImage,
            appimage.clone(),
            Some(format!("{target}-{}", BundleTarget::AppImage.name())),
            vec![UPDATED_EXIT_CODE, UP_TO_DATE_EXIT_CODE],
        ),
        // no update
        (
            BundleTarget::AppImage,
            appimage,
            None,
            vec![ERROR_EXIT_CODE],
        ),
    ]
}

/// The cases of a Linux package: an update announced under the generic target and one under the
/// package specific target. There is no "no update" case, the AppImage one covers that path.
#[cfg(target_os = "linux")]
fn package_test_cases(
    root_dir: &Path,
    version: &str,
    target: String,
    bundle_target: BundleTarget,
) -> Vec<TestCase> {
    let package = bundle_target.bundle_path(root_dir, version);
    vec![
        // update using fallback
        (
            bundle_target,
            package.clone(),
            Some(target.clone()),
            vec![UPDATED_EXIT_CODE, UP_TO_DATE_EXIT_CODE],
        ),
        // update using full name
        (
            bundle_target,
            package,
            Some(format!("{target}-{}", bundle_target.name())),
            vec![UPDATED_EXIT_CODE, UP_TO_DATE_EXIT_CODE],
        ),
    ]
}

#[cfg(target_os = "macos")]
fn test_cases(root_dir: &Path, _version: &str, target: String) -> Vec<TestCase> {
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
fn test_cases(root_dir: &Path, version: &str, target: String) -> Vec<TestCase> {
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
#[ignore = "needs the tauri CLI, a display and minutes of build time; the integration tests workflow runs it with --ignored"]
fn update_app() {
    let _lock = BUILD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let target =
        tauri_plugin_updater::target().expect("running updater test in an unsupported platform");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.join("../../../..");

    for config in [
        Config {
            version: "1.0.0",
            bundle: BundleConfig {
                create_updater_artifacts: Updater::Bool(true),
            },
            plugins: None,
        },
        Config {
            version: "1.0.0",
            bundle: BundleConfig {
                create_updater_artifacts: Updater::String(V1Compatible::V1Compatible),
            },
            plugins: None,
        },
    ] {
        let cases = test_cases(&root_dir, "1.0.0", target.clone());
        run_update_cases(&manifest_dir, &root_dir, &target, config, cases);
    }
}

/// The updater installs a deb through `dpkg` and an rpm through `rpm`, so this runs the installed
/// app and lets the update replace it system wide. It only runs where that can happen without any
/// prompt (see `BundleTarget::installable`) and removes the package when done.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "needs the tauri CLI, a display and minutes of build time; the integration tests workflow runs it with --ignored"]
fn update_app_deb() {
    update_linux_package(BundleTarget::Deb);
}

/// See `update_app_deb`.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "needs the tauri CLI, a display and minutes of build time; the integration tests workflow runs it with --ignored"]
fn update_app_rpm() {
    update_linux_package(BundleTarget::Rpm);
}

#[cfg(target_os = "linux")]
fn update_linux_package(bundle_target: BundleTarget) {
    let _lock = BUILD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    if let Err(reason) = bundle_target.installable() {
        eprintln!(
            "skipping the {} update test: {reason}",
            bundle_target.name()
        );
        return;
    }

    let target =
        tauri_plugin_updater::target().expect("running updater test in an unsupported platform");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.join("../../../..");

    // the legacy v1 updater never supported these packages, so only the v2 artifacts are tested
    let config = Config {
        version: "1.0.0",
        bundle: BundleConfig {
            create_updater_artifacts: Updater::Bool(true),
        },
        plugins: None,
    };
    let cases = package_test_cases(&root_dir, "1.0.0", target.clone(), bundle_target);
    run_update_cases(&manifest_dir, &root_dir, &target, config, cases);
}

/// Bundles 1.0.0 as each case's target and serves it, then bundles 0.1.0, runs it once per expected
/// exit code and checks what it reports.
fn run_update_cases(
    manifest_dir: &Path,
    root_dir: &Path,
    target: &str,
    mut config: Config,
    cases: Vec<TestCase>,
) {
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

    for (bundle_target, out_bundle_path, update_platform, status_checks) in cases {
        // bundle app update
        config.version = "1.0.0";
        build_app(manifest_dir, &config, Some(bundle_target));

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
                            version: "1.0.0".into(),
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

        config.version = "0.1.0";

        // bundle initial app version; Linux and macOS run the bundle itself, so it cannot be
        // skipped there
        build_app(
            manifest_dir,
            &config,
            if cfg!(windows) {
                None
            } else {
                Some(bundle_target)
            },
        );

        let app_path = stage_app_under_test(root_dir, target, bundle_target);

        for expected_exit_code in status_checks {
            let mut binary_cmd = if cfg!(target_os = "macos") {
                Command::new(app_path.join("Contents/MacOS/app-updater"))
            } else if cfg!(target_os = "linux")
                && std::env::var("CI").map(|v| v == "true").unwrap_or_default()
            {
                let mut c = Command::new("xvfb-run");
                c.arg("--auto-servernum").arg(&app_path);
                c
            } else {
                Command::new(&app_path)
            };

            binary_cmd.env("TARGET", bundle_target.name());

            let status = binary_cmd
                .status()
                .unwrap_or_else(|e| panic!("failed to run {}: {e}", app_path.display()));
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

        #[cfg(target_os = "linux")]
        {
            remove_package(bundle_target);
        }

        // graceful shutdown
        server.unblock();
    }
}

const SIGNED_VERSION_PORT: u16 = 3008;
/// Fragment of `Error::SignedVersionMismatch`. The app prints the updater error before exiting,
/// and a rejected update exits with the same code as a failed install, so the message is what
/// tells them apart.
const MISMATCH_ERROR: &str = "was signed for version";

struct ServedUpdate {
    version: String,
}

fn app_binary(root_dir: &Path) -> PathBuf {
    root_dir.join(if cfg!(windows) {
        "target/release/app-updater.exe"
    } else {
        "target/release/app-updater"
    })
}

/// Runs the app against the update server, restoring the binary from `pristine` first.
///
/// A case that clears the version check goes on to actually install, and on Linux installing means
/// writing the downloaded bytes over the running executable. Every case therefore has to start
/// from an untouched binary, or the first one that passes leaves the update behind as the app.
fn run_app(root_dir: &Path, pristine: &Path) -> String {
    let binary = app_binary(root_dir);
    std::fs::copy(pristine, &binary).expect("failed to restore the app binary");

    let mut command = if cfg!(target_os = "linux")
        && std::env::var("CI").map(|v| v == "true").unwrap_or_default()
    {
        let mut c = Command::new("xvfb-run");
        c.arg("--auto-servernum").arg(&binary);
        c
    } else {
        Command::new(&binary)
    };

    let output = command.output().expect("failed to run app");
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The update endpoint response is not signed, so its `version` field alone does not prove which
/// release the `url` and `signature` point at. This bundles a genuine 1.0.0 release and then varies
/// only the version the endpoint announces for it, which is the shape of a forced downgrade: a
/// tampered response pairing a new version number with an older release's signature.
///
/// The check runs in `Update::download`, before anything is installed, so a rejected case never
/// reaches the installer while an accepted one does. Both exit non-zero, which is why these assert
/// on the error message rather than the exit code.
#[test]
#[ignore = "needs the tauri CLI, a display and minutes of build time; the integration tests workflow runs it with --ignored"]
fn update_validates_signed_version() {
    let _lock = BUILD_LOCK.lock().unwrap_or_else(|e| e.into_inner());

    let target =
        tauri_plugin_updater::target().expect("running updater test in an unsupported platform");
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let root_dir = manifest_dir.join("../../../..");

    // bundle the release the endpoint will point at; the CLI records 1.0.0 in its signature
    let bundle_target = BundleTarget::default();
    build_app(
        &manifest_dir,
        &Config {
            version: "1.0.0",
            bundle: BundleConfig {
                create_updater_artifacts: Updater::Bool(true),
            },
            plugins: None,
        },
        Some(bundle_target),
    );

    let out_bundle_path = test_cases(&root_dir, "1.0.0", target.clone())
        .first()
        .unwrap()
        .1
        .clone();
    let updater_extension = {
        let bundle_ext = out_bundle_path
            .extension()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        if cfg!(target_os = "macos") {
            format!("{bundle_ext}.tar.gz")
        } else {
            bundle_ext
        }
    };
    let signature =
        std::fs::read_to_string(out_bundle_path.with_extension(format!("{updater_extension}.sig")))
            .expect("failed to read signature file");

    // move it aside so building the running app cannot clobber it
    let out_updater_path = out_bundle_path.with_extension(&updater_extension);
    let updater_path = root_dir.join(format!(
        "target/release/{}",
        out_updater_path.file_name().unwrap().to_str().unwrap()
    ));
    std::fs::rename(&out_updater_path, &updater_path).expect("failed to rename bundle");

    let served = Arc::new(Mutex::new(ServedUpdate {
        version: "1.0.0".into(),
    }));

    let server = Arc::new(
        tiny_http::Server::http(format!("localhost:{SIGNED_VERSION_PORT}"))
            .expect("failed to start updater server"),
    );

    let server_ = server.clone();
    let served_ = served.clone();
    let target_ = target.clone();
    let updater_path_ = updater_path.clone();
    std::thread::spawn(move || {
        for request in server_.incoming_requests() {
            match request.url() {
                "/" => {
                    let mut platforms = HashMap::new();
                    platforms.insert(
                        target_.clone(),
                        PlatformUpdate {
                            // always the genuine 1.0.0 signature; only the version above it moves
                            signature: signature.clone(),
                            url: format!("http://localhost:{SIGNED_VERSION_PORT}/download"),
                            with_elevated_task: false,
                        },
                    );

                    let body = serde_json::to_vec(&Update {
                        version: served_.lock().unwrap().version.clone(),
                        date: time::OffsetDateTime::now_utc()
                            .format(&time::format_description::well_known::Rfc3339)
                            .unwrap(),
                        platforms,
                    })
                    .unwrap();
                    let len = body.len();
                    let _ = request.respond(tiny_http::Response::new(
                        tiny_http::StatusCode(200),
                        Vec::new(),
                        std::io::Cursor::new(body),
                        Some(len),
                        None,
                    ));
                }
                "/download" => {
                    let _ = request.respond(tiny_http::Response::from_file(
                        File::open(&updater_path_).unwrap_or_else(|_| {
                            panic!("failed to open updater bundle {}", updater_path_.display())
                        }),
                    ));
                }
                _ => (),
            }
        }
    });

    // `requireSignedVersion` is baked in by `generate_context!`, so each value needs its own build
    let pristine = root_dir.join("target/release/app-updater.pristine");
    let build = |require_signed_version: bool| {
        build_app(
            &manifest_dir,
            &Config {
                version: "0.1.0",
                bundle: BundleConfig {
                    create_updater_artifacts: Updater::Bool(true),
                },
                plugins: Some(serde_json::json!({
                    "updater": {
                        "endpoints": [format!("http://localhost:{SIGNED_VERSION_PORT}")],
                        "requireSignedVersion": require_signed_version,
                    }
                })),
            },
            None,
        );
        std::fs::copy(app_binary(&root_dir), &pristine)
            .expect("failed to keep a pristine copy of the app binary");
    };

    let check = |announced: &str, expected: Option<&str>, unexpected: Option<&str>| {
        served.lock().unwrap().version = announced.to_string();
        let output = run_app(&root_dir, &pristine);

        if let Some(expected) = expected {
            assert!(
                output.contains(expected),
                "expected {expected:?} when announcing {announced} for a 1.0.0 signature, got: {output}"
            );
        }
        if let Some(unexpected) = unexpected {
            assert!(
                !output.contains(unexpected),
                "unexpected {unexpected:?} when announcing {announced} for a 1.0.0 signature, got: {output}"
            );
        }
    };

    build(true);

    // the rollback this exists to stop: a 1.0.0 release dressed up as a newer one
    check("1.5.0", Some(MISMATCH_ERROR), None);
    // the announced version is what the artifact was signed for, so it must go through
    check("1.0.0", None, Some(MISMATCH_ERROR));

    build(false);

    // a signature that names a version is held to it whether or not the option is on
    check("1.5.0", Some(MISMATCH_ERROR), None);

    server.unblock();

    // leave a runnable binary behind rather than whichever update was installed last
    let _ = std::fs::copy(&pristine, app_binary(&root_dir));
    let _ = std::fs::remove_file(&pristine);
}
