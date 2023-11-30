// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    collections::HashMap,
    io::{Cursor, Read},
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};

use base64::Engine;
use futures_util::StreamExt;
use http::HeaderName;
use minisign_verify::{PublicKey, Signature};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, StatusCode,
};
use semver::Version;
use serde::{de::Error as DeError, Deserialize, Deserializer, Serialize};
use tauri::utils::{config::UpdaterConfig, platform::current_exe};
use time::OffsetDateTime;
use url::Url;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ReleaseManifestPlatform {
    /// Download URL for the platform
    pub url: Url,
    /// Signature for the platform
    pub signature: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(untagged)]
pub enum RemoteReleaseInner {
    Dynamic(ReleaseManifestPlatform),
    Static {
        platforms: HashMap<String, ReleaseManifestPlatform>,
    },
}

/// Information about a release returned by the remote update server.
///
/// This type can have one of two shapes: Server Format (Dynamic Format) and Static Format.
#[derive(Debug, Clone)]
pub struct RemoteRelease {
    /// Version to install.
    pub version: Version,
    /// Release notes.
    pub notes: Option<String>,
    /// Release date.
    pub pub_date: Option<OffsetDateTime>,
    /// Release data.
    pub data: RemoteReleaseInner,
}

impl RemoteRelease {
    /// The release's download URL for the given target.
    pub fn download_url(&self, target: &str) -> Result<&Url> {
        match self.data {
            RemoteReleaseInner::Dynamic(ref platform) => Ok(&platform.url),
            RemoteReleaseInner::Static { ref platforms } => platforms
                .get(target)
                .map_or(Err(Error::TargetNotFound(target.to_string())), |p| {
                    Ok(&p.url)
                }),
        }
    }

    /// The release's signature for the given target.
    pub fn signature(&self, target: &str) -> Result<&String> {
        match self.data {
            RemoteReleaseInner::Dynamic(ref platform) => Ok(&platform.signature),
            RemoteReleaseInner::Static { ref platforms } => platforms
                .get(target)
                .map_or(Err(Error::TargetNotFound(target.to_string())), |platform| {
                    Ok(&platform.signature)
                }),
        }
    }
}

pub struct UpdaterBuilder {
    current_version: Version,
    config: crate::Config,
    updater_config: UpdaterConfig,
    version_comparator: Option<Box<dyn Fn(Version, RemoteRelease) -> bool + Send + Sync>>,
    executable_path: Option<PathBuf>,
    target: Option<String>,
    endpoints: Option<Vec<Url>>,
    headers: HeaderMap,
    timeout: Option<Duration>,
    installer_args: Option<Vec<String>>,
}

impl UpdaterBuilder {
    pub fn new(
        current_version: Version,
        config: crate::Config,
        updater_config: UpdaterConfig,
    ) -> Self {
        Self {
            current_version,
            config,
            updater_config,
            version_comparator: None,
            executable_path: None,
            target: None,
            endpoints: None,
            headers: Default::default(),
            timeout: None,
            installer_args: None,
        }
    }

    pub fn version_comparator<F: Fn(Version, RemoteRelease) -> bool + Send + Sync + 'static>(
        mut self,
        f: F,
    ) -> Self {
        self.version_comparator = Some(Box::new(f));
        self
    }

    pub fn target(mut self, target: impl Into<String>) -> Self {
        self.target.replace(target.into());
        self
    }

    pub fn endpoints(mut self, endpoints: Vec<Url>) -> Self {
        self.endpoints.replace(endpoints);
        self
    }

    pub fn executable_path<P: AsRef<Path>>(mut self, p: P) -> Self {
        self.executable_path.replace(p.as_ref().into());
        self
    }

    pub fn header<K, V>(mut self, key: K, value: V) -> Result<Self>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        let key: std::result::Result<HeaderName, http::Error> = key.try_into().map_err(Into::into);
        let value: std::result::Result<HeaderValue, http::Error> =
            value.try_into().map_err(Into::into);
        self.headers.insert(key?, value?);

        Ok(self)
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn installer_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.installer_args
            .replace(args.into_iter().map(Into::into).collect());
        self
    }

    pub fn build(self) -> Result<Updater> {
        let endpoints = self
            .endpoints
            .unwrap_or_else(|| self.config.endpoints.into_iter().map(|e| e.0).collect());

        if endpoints.is_empty() {
            return Err(Error::EmptyEndpoints);
        };

        let arch = get_updater_arch().ok_or(Error::UnsupportedArch)?;
        let (target, json_target) = if let Some(target) = self.target {
            (target.clone(), target)
        } else {
            let target = get_updater_target().ok_or(Error::UnsupportedOs)?;
            (target.to_string(), format!("{target}-{arch}"))
        };

        let executable_path = self.executable_path.clone().unwrap_or(current_exe()?);

        // Get the extract_path from the provided executable_path
        let extract_path = if cfg!(target_os = "linux") {
            executable_path
        } else {
            extract_path_from_executable(&executable_path)?
        };

        Ok(Updater {
            config: self.updater_config,
            current_version: self.current_version,
            version_comparator: self.version_comparator,
            timeout: self.timeout,
            endpoints,
            installer_args: self.installer_args.unwrap_or(self.config.installer_args),
            arch,
            target,
            json_target,
            headers: self.headers,
            extract_path,
        })
    }
}

pub struct Updater {
    config: UpdaterConfig,
    current_version: Version,
    version_comparator: Option<Box<dyn Fn(Version, RemoteRelease) -> bool + Send + Sync>>,
    timeout: Option<Duration>,
    endpoints: Vec<Url>,
    #[allow(dead_code)]
    installer_args: Vec<String>,
    arch: &'static str,
    // The `{{target}}` variable we replace in the endpoint
    target: String,
    // The value we search if the updater server returns a JSON with the `platforms` object
    json_target: String,
    headers: HeaderMap,
    extract_path: PathBuf,
}

impl Updater {
    pub async fn check(&self) -> Result<Option<Update>> {
        // we want JSON only
        let mut headers = self.headers.clone();
        headers.insert("Accept", HeaderValue::from_str("application/json").unwrap());

        // Set SSL certs for linux if they aren't available.
        #[cfg(target_os = "linux")]
        {
            if std::env::var_os("SSL_CERT_FILE").is_none() {
                std::env::set_var("SSL_CERT_FILE", "/etc/ssl/certs/ca-certificates.crt");
            }
            if std::env::var_os("SSL_CERT_DIR").is_none() {
                std::env::set_var("SSL_CERT_DIR", "/etc/ssl/certs");
            }
        }

        let mut remote_release: Option<RemoteRelease> = None;
        let mut last_error: Option<Error> = None;
        for url in &self.endpoints {
            // replace {{current_version}}, {{target}} and {{arch}} in the provided URL
            // this is useful if we need to query example
            // https://releases.myapp.com/update/{{target}}/{{arch}}/{{current_version}}
            // will be translated into ->
            // https://releases.myapp.com/update/darwin/aarch64/1.0.0
            // The main objective is if the update URL is defined via the Cargo.toml
            // the URL will be generated dynamically
            let url: Url = url
                .to_string()
                // url::Url automatically url-encodes the path components
                .replace(
                    "%7B%7Bcurrent_version%7D%7D",
                    &self.current_version.to_string(),
                )
                .replace("%7B%7Btarget%7D%7D", &self.target)
                .replace("%7B%7Barch%7D%7D", self.arch)
                // but not query parameters
                .replace("{{current_version}}", &self.current_version.to_string())
                .replace("{{target}}", &self.target)
                .replace("{{arch}}", self.arch)
                .parse()?;

            let mut request = Client::new().get(url).headers(headers.clone());
            if let Some(timeout) = self.timeout {
                request = request.timeout(timeout);
            }
            let response = request.send().await;

            if let Ok(res) = response {
                if res.status().is_success() {
                    // no updates found!
                    if StatusCode::NO_CONTENT == res.status() {
                        return Ok(None);
                    };

                    match serde_json::from_value::<RemoteRelease>(res.json().await?)
                        .map_err(Into::into)
                    {
                        Ok(release) => {
                            last_error = None;
                            remote_release = Some(release);
                            // we found a relase, break the loop
                            break;
                        }
                        Err(err) => last_error = Some(err),
                    }
                }
            }
        }

        // Last error is cleaned on success.
        // Shouldn't be triggered if we had a successfull call
        if let Some(error) = last_error {
            return Err(error);
        }

        // Extracted remote metadata
        let release = remote_release.ok_or(Error::ReleaseNotFound)?;

        let should_update = match self.version_comparator.as_ref() {
            Some(comparator) => comparator(self.current_version.clone(), release.clone()),
            None => release.version > self.current_version,
        };

        let update = if should_update {
            Some(Update {
                current_version: self.current_version.to_string(),
                config: self.config.clone(),
                target: self.target.clone(),
                extract_path: self.extract_path.clone(),
                installer_args: self.installer_args.clone(),
                version: release.version.to_string(),
                date: release.pub_date,
                download_url: release.download_url(&self.json_target)?.to_owned(),
                body: release.notes.clone(),
                signature: release.signature(&self.json_target)?.to_owned(),
                timeout: self.timeout,
                headers: self.headers.clone(),
            })
        } else {
            None
        };

        Ok(update)
    }
}

#[derive(Debug, Clone)]
pub struct Update {
    config: UpdaterConfig,
    /// Update description
    pub body: Option<String>,
    /// Version used to check for update
    pub current_version: String,
    /// Version announced
    pub version: String,
    /// Update publish date
    pub date: Option<OffsetDateTime>,
    /// Target
    pub target: String,
    /// Extract path
    #[allow(unused)]
    extract_path: PathBuf,
    #[allow(unused)]
    installer_args: Vec<String>,
    /// Download URL announced
    pub download_url: Url,
    /// Signature announced
    pub signature: String,
    /// Request timeout
    pub timeout: Option<Duration>,
    /// Request headers
    pub headers: HeaderMap,
}

impl Update {
    /// Downloads the updater package, verifies it then return it as bytes.
    ///
    /// Use [`Update::install`] to install it
    pub async fn download<C: Fn(usize, Option<u64>), D: FnOnce()>(
        &self,
        on_chunk: C,
        on_download_finish: D,
    ) -> Result<Vec<u8>> {
        // set our headers
        let mut headers = self.headers.clone();
        headers.insert(
            "Accept",
            HeaderValue::from_str("application/octet-stream").unwrap(),
        );
        headers.insert(
            "User-Agent",
            HeaderValue::from_str("tauri-updater").unwrap(),
        );

        let mut request = Client::new()
            .get(self.download_url.clone())
            .headers(headers);
        if let Some(timeout) = self.timeout {
            request = request.timeout(timeout);
        }
        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(Error::Network(format!(
                "Download request failed with status: {}",
                response.status()
            )));
        }

        let content_length: Option<u64> = response
            .headers()
            .get("Content-Length")
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse().ok());

        let mut buffer = Vec::new();

        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let bytes = chunk.as_ref().to_vec();
            on_chunk(bytes.len(), content_length);
            buffer.extend(bytes);
        }

        on_download_finish();

        let mut update_buffer = Cursor::new(&buffer);

        verify_signature(&mut update_buffer, &self.signature, &self.config.pubkey)?;

        Ok(buffer)
    }

    /// Installs the updater package downloaded by [`Update::download`]
    pub fn install(&self, bytes: Vec<u8>) -> Result<()> {
        self.install_inner(bytes)
    }

    /// Downloads and installs the updater package
    pub async fn download_and_install<C: Fn(usize, Option<u64>), D: FnOnce()>(
        &self,
        on_chunk: C,
        on_download_finish: D,
    ) -> Result<()> {
        let bytes = self.download(on_chunk, on_download_finish).await?;
        self.install(bytes)
    }

    #[cfg(mobile)]
    fn install_inner(&self, bytes: Vec<u8>) -> Result<()> {
        Ok(())
    }

    // Windows
    //
    // ### Expected structure:
    // ├── [AppName]_[version]_x64.msi.zip          # ZIP generated by tauri-bundler
    // │   └──[AppName]_[version]_x64.msi           # Application MSI
    // ├── [AppName]_[version]_x64-setup.exe.zip          # ZIP generated by tauri-bundler
    // │   └──[AppName]_[version]_x64-setup.exe           # NSIS installer
    // └── ...
    //
    // ## MSI
    // Update server can provide a MSI for Windows. (Generated with tauri-bundler from *Wix*)
    // To replace current version of the application. In later version we'll offer
    // incremental update to push specific binaries.
    //
    // ## EXE
    // Update server can provide a custom EXE (installer) who can run any task.
    #[cfg(windows)]
    fn install_inner(&self, bytes: Vec<u8>) -> Result<()> {
        use std::{ffi::OsStr, fs, process::Command};

        // FIXME: We need to create a memory buffer with the MSI and then run it.
        //        (instead of extracting the MSI to a temp path)
        //
        // The tricky part is the MSI need to be exposed and spawned so the memory allocation
        // shouldn't drop but we should be able to pass the reference so we can drop it once the installation
        // is done, otherwise we have a huge memory leak.

        let archive = Cursor::new(bytes);

        let tmp_dir = tempfile::Builder::new().tempdir()?.into_path();

        // extract the buffer to the tmp_dir
        // we extract our signed archive into our final directory without any temp file
        let mut extractor = zip::ZipArchive::new(archive)?;

        // extract the msi
        extractor.extract(&tmp_dir)?;

        let paths = fs::read_dir(&tmp_dir)?;

        let system_root = std::env::var("SYSTEMROOT");
        let powershell_path = system_root.as_ref().map_or_else(
            |_| "powershell.exe".to_string(),
            |p| format!("{p}\\System32\\WindowsPowerShell\\v1.0\\powershell.exe"),
        );

        for path in paths {
            let found_path = path?.path();
            // we support 2 type of files exe & msi for now
            // If it's an `exe` we expect an installer not a runtime.
            if found_path.extension() == Some(OsStr::new("exe")) {
                // we need to wrap the installer path in quotes for Start-Process
                let mut installer_arg = std::ffi::OsString::new();
                installer_arg.push("\"");
                installer_arg.push(&found_path);
                installer_arg.push("\"");

                // Run the installer
                Command::new(powershell_path)
                    .args(["-NoProfile", "-WindowStyle", "Hidden"])
                    .args(["Start-Process"])
                    .arg(installer_arg)
                    .arg("-ArgumentList")
                    .arg(
                        [
                            self.config.windows.install_mode.nsis_args(),
                            self.installer_args
                                .iter()
                                .map(AsRef::as_ref)
                                .collect::<Vec<_>>()
                                .as_slice(),
                        ]
                        .concat()
                        .join(", "),
                    )
                    .spawn()
                    .expect("installer failed to start");

                std::process::exit(0);
            } else if found_path.extension() == Some(OsStr::new("msi")) {
                // we need to wrap the current exe path in quotes for Start-Process
                let mut current_exe_arg = std::ffi::OsString::new();
                current_exe_arg.push("\"");
                current_exe_arg.push(current_exe()?);
                current_exe_arg.push("\"");

                let mut msi_path_arg = std::ffi::OsString::new();
                msi_path_arg.push("\"\"\"");
                msi_path_arg.push(&found_path);
                msi_path_arg.push("\"\"\"");

                let msiexec_args = self
                    .config
                    .windows
                    .install_mode
                    .msiexec_args()
                    .iter()
                    .map(|p| p.to_string())
                    .collect::<Vec<String>>();

                // run the installer and relaunch the application
                let powershell_install_res = Command::new(powershell_path)
                    .args(["-NoProfile", "-WindowStyle", "Hidden"])
                    .args([
                        "Start-Process",
                        "-Wait",
                        "-FilePath",
                        "$env:SYSTEMROOT\\System32\\msiexec.exe",
                        "-ArgumentList",
                    ])
                    .arg("/i,")
                    .arg(&msi_path_arg)
                    .arg(format!(", {}, /promptrestart;", msiexec_args.join(", ")))
                    .arg("Start-Process")
                    .arg(current_exe_arg)
                    .spawn();
                if powershell_install_res.is_err() {
                    // fallback to running msiexec directly - relaunch won't be available
                    // we use this here in case powershell fails in an older machine somehow
                    let msiexec_path = system_root.as_ref().map_or_else(
                        |_| "msiexec.exe".to_string(),
                        |p| format!("{p}\\System32\\msiexec.exe"),
                    );
                    let _ = Command::new(msiexec_path)
                        .arg("/i")
                        .arg(msi_path_arg)
                        .args(msiexec_args)
                        .arg("/promptrestart")
                        .spawn();
                }

                std::process::exit(0);
            }
        }

        Ok(())
    }

    // Linux (AppImage)
    //
    // ### Expected structure:
    // ├── [AppName]_[version]_amd64.AppImage.tar.gz    # GZ generated by tauri-bundler
    // │   └──[AppName]_[version]_amd64.AppImage        # Application AppImage
    // └── ...
    //
    // We should have an AppImage already installed to be able to copy and install
    // the extract_path is the current AppImage path
    // tmp_dir is where our new AppImage is found
    #[cfg(any(
        target_os = "linux",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    fn install_inner(&self, bytes: Vec<u8>) -> Result<()> {
        use flate2::read::GzDecoder;
        use std::{
            ffi::OsStr,
            os::unix::fs::{MetadataExt, PermissionsExt},
        };
        let archive = Cursor::new(bytes);
        let extract_path_metadata = self.extract_path.metadata()?;

        let tmp_dir_locations = vec![
            Box::new(|| Some(std::env::temp_dir())) as Box<dyn FnOnce() -> Option<PathBuf>>,
            Box::new(dirs_next::cache_dir),
            Box::new(|| Some(self.extract_path.parent().unwrap().to_path_buf())),
        ];

        for tmp_dir_location in tmp_dir_locations {
            if let Some(tmp_dir_location) = tmp_dir_location() {
                let tmp_dir = tempfile::Builder::new()
                    .prefix("tauri_current_app")
                    .tempdir_in(tmp_dir_location)?;
                let tmp_dir_metadata = tmp_dir.path().metadata()?;

                if extract_path_metadata.dev() == tmp_dir_metadata.dev() {
                    let mut perms = tmp_dir_metadata.permissions();
                    perms.set_mode(0o700);
                    std::fs::set_permissions(tmp_dir.path(), perms)?;

                    let tmp_app_image = &tmp_dir.path().join("current_app.AppImage");

                    // create a backup of our current app image
                    std::fs::rename(&self.extract_path, tmp_app_image)?;

                    // extract the buffer to the tmp_dir
                    // we extract our signed archive into our final directory without any temp file
                    let decoder = GzDecoder::new(archive);
                    let mut archive = tar::Archive::new(decoder);
                    for mut entry in archive.entries()?.flatten() {
                        if let Ok(path) = entry.path() {
                            if path.extension() == Some(OsStr::new("AppImage")) {
                                // if something went wrong during the extraction, we should restore previous app
                                if let Err(err) = entry.unpack(&self.extract_path) {
                                    std::fs::rename(tmp_app_image, &self.extract_path)?;
                                    return Err(err.into());
                                }
                                // early finish we have everything we need here
                                return Ok(());
                            }
                        }
                    }
                    // if we have not returned early we should restore the backup
                    std::fs::rename(tmp_app_image, &self.extract_path)?;

                    return Err(Error::BinaryNotFoundInAcrhive);
                }
            }
        }

        Err(Error::TempDirNotOnSameMountPoint)
    }

    // MacOS
    //
    // ### Expected structure:
    // ├── [AppName]_[version]_x64.app.tar.gz       # GZ generated by tauri-bundler
    // │   └──[AppName].app                         # Main application
    // │      └── Contents                          # Application contents...
    // │          └── ...
    // └── ...
    #[cfg(target_os = "macos")]
    fn install_inner(&self, bytes: Vec<u8>) -> Result<()> {
        use flate2::read::GzDecoder;

        let cursor = Cursor::new(bytes);
        let mut extracted_files: Vec<PathBuf> = Vec::new();

        // the first file in the tar.gz will always be
        // <app_name>/Contents
        let tmp_dir = tempfile::Builder::new()
            .prefix("tauri_current_app")
            .tempdir()?;

        // create backup of our current app
        std::fs::rename(&self.extract_path, tmp_dir.path())?;

        let decoder = GzDecoder::new(cursor);
        let mut archive = tar::Archive::new(decoder);

        std::fs::create_dir(&self.extract_path)?;

        for entry in archive.entries()? {
            let mut entry = entry?;

            // skip the first folder (should be the app name)
            let collected_path: PathBuf = entry.path()?.iter().skip(1).collect();
            let extraction_path = &self.extract_path.join(collected_path);

            // if something went wrong during the extraction, we should restore previous app
            if let Err(err) = entry.unpack(extraction_path) {
                for file in extracted_files.iter().rev() {
                    // delete all the files we extracted
                    if file.is_dir() {
                        std::fs::remove_dir(file)?;
                    } else {
                        std::fs::remove_file(file)?;
                    }
                }
                std::fs::rename(tmp_dir.path(), &self.extract_path)?;
                return Err(err.into());
            }

            extracted_files.push(extraction_path.to_path_buf());
        }

        let _ = std::process::Command::new("touch")
            .arg(&self.extract_path)
            .status();

        Ok(())
    }
}

/// Gets the target string used on the updater.
pub fn target() -> Option<String> {
    if let (Some(target), Some(arch)) = (get_updater_target(), get_updater_arch()) {
        Some(format!("{target}-{arch}"))
    } else {
        None
    }
}

pub(crate) fn get_updater_target() -> Option<&'static str> {
    if cfg!(target_os = "linux") {
        Some("linux")
    } else if cfg!(target_os = "macos") {
        // TODO shouldn't this be macos instead?
        Some("darwin")
    } else if cfg!(target_os = "windows") {
        Some("windows")
    } else {
        None
    }
}

pub(crate) fn get_updater_arch() -> Option<&'static str> {
    if cfg!(target_arch = "x86") {
        Some("i686")
    } else if cfg!(target_arch = "x86_64") {
        Some("x86_64")
    } else if cfg!(target_arch = "arm") {
        Some("armv7")
    } else if cfg!(target_arch = "aarch64") {
        Some("aarch64")
    } else {
        None
    }
}

pub fn extract_path_from_executable(executable_path: &Path) -> Result<PathBuf> {
    // Return the path of the current executable by default
    // Example C:\Program Files\My App\
    let extract_path = executable_path
        .parent()
        .map(PathBuf::from)
        .ok_or(Error::FailedToDetermineExtractPath)?;

    // MacOS example binary is in /Applications/TestApp.app/Contents/MacOS/myApp
    // We need to get /Applications/<app>.app
    // TODO(lemarier): Need a better way here
    // Maybe we could search for <*.app> to get the right path
    #[cfg(target_os = "macos")]
    if extract_path
        .display()
        .to_string()
        .contains("Contents/MacOS")
    {
        return extract_path
            .parent()
            .map(PathBuf::from)
            .ok_or(Error::FailedToDetermineExtractPath)?
            .parent()
            .map(PathBuf::from)
            .ok_or(Error::FailedToDetermineExtractPath);
    }

    Ok(extract_path)
}

impl<'de> Deserialize<'de> for RemoteRelease {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct InnerRemoteRelease {
            #[serde(alias = "name", deserialize_with = "parse_version")]
            version: Version,
            notes: Option<String>,
            pub_date: Option<String>,
            platforms: Option<HashMap<String, ReleaseManifestPlatform>>,
            // dynamic platform response
            url: Option<Url>,
            signature: Option<String>,
        }

        let release = InnerRemoteRelease::deserialize(deserializer)?;

        let pub_date = if let Some(date) = release.pub_date {
            Some(
                OffsetDateTime::parse(&date, &time::format_description::well_known::Rfc3339)
                    .map_err(|e| DeError::custom(format!("invalid value for `pub_date`: {e}")))?,
            )
        } else {
            None
        };

        Ok(RemoteRelease {
            version: release.version,
            notes: release.notes,
            pub_date,
            data: if let Some(platforms) = release.platforms {
                RemoteReleaseInner::Static { platforms }
            } else {
                RemoteReleaseInner::Dynamic(ReleaseManifestPlatform {
                    url: release.url.ok_or_else(|| {
                        DeError::custom("the `url` field was not set on the updater response")
                    })?,
                    signature: release.signature.ok_or_else(|| {
                        DeError::custom("the `signature` field was not set on the updater response")
                    })?,
                })
            },
        })
    }
}

fn parse_version<'de, D>(deserializer: D) -> std::result::Result<Version, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let str = String::deserialize(deserializer)?;

    Version::from_str(str.trim_start_matches('v')).map_err(serde::de::Error::custom)
}

// Validate signature
// need to be public because its been used
// by our tests in the bundler
//
// NOTE: The buffer position is not reset.
pub fn verify_signature<R>(
    archive_reader: &mut R,
    release_signature: &str,
    pub_key: &str,
) -> Result<bool>
where
    R: Read,
{
    // we need to convert the pub key
    let pub_key_decoded = base64_to_string(pub_key)?;
    let public_key = PublicKey::decode(&pub_key_decoded)?;
    let signature_base64_decoded = base64_to_string(release_signature)?;
    let signature = Signature::decode(&signature_base64_decoded)?;

    // read all bytes until EOF in the buffer
    let mut data = Vec::new();
    archive_reader.read_to_end(&mut data)?;

    // Validate signature or bail out
    public_key.verify(&data, &signature, true)?;
    Ok(true)
}

fn base64_to_string(base64_string: &str) -> Result<String> {
    let decoded_string = &base64::engine::general_purpose::STANDARD.decode(base64_string)?;
    let result = std::str::from_utf8(decoded_string)
        .map_err(|_| Error::SignatureUtf8(base64_string.into()))?
        .to_string();
    Ok(result)
}
