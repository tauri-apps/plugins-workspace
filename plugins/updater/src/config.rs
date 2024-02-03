// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Deserializer};
use url::Url;

/// Updater configuration.
#[derive(Debug, Clone, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub endpoints: Vec<UpdaterEndpoint>,
    /// Signature public key.
    pub pubkey: String,
    /// Additional arguments given to the NSIS or WiX installer.
    #[serde(default, alias = "installer-args")]
    pub installer_args: Vec<String>,
    /// The Windows configuration for the updater.
    #[serde(default)]
    pub windows: UpdaterWindowsConfig,
}

/// Install modes for the Windows update.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum WindowsUpdateInstallMode {
    /// Specifies there's a basic UI during the installation process, including a final dialog box at the end.
    BasicUi,
    /// The quiet mode means there's no user interaction required.
    /// Requires admin privileges if the installer does.
    Quiet,
    /// Specifies unattended mode, which means the installation only shows a progress bar.
    Passive,
    // to add more modes, we need to check if the updater relaunch makes sense
    // i.e. for a full UI mode, the user can also mark the installer to start the app
}

impl Default for WindowsUpdateInstallMode {
    fn default() -> Self {
        Self::Passive
    }
}

impl<'de> Deserialize<'de> for WindowsUpdateInstallMode {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        match s.to_lowercase().as_str() {
            "basicui" => Ok(Self::BasicUi),
            "quiet" => Ok(Self::Quiet),
            "passive" => Ok(Self::Passive),
            _ => Err(serde::de::Error::custom(format!(
                "unknown update install mode '{s}'"
            ))),
        }
    }
}

impl WindowsUpdateInstallMode {
    /// Returns the associated nsis arguments.
    pub fn nsis_args(&self) -> &'static [&'static str] {
        match self {
            Self::Passive => &["/P", "/R"],
            Self::Quiet => &["/S", "/R"],
            _ => &[],
        }
    }

    /// Returns the associated `msiexec.exe` arguments.
    pub fn msiexec_args(&self) -> &'static [&'static str] {
        match self {
            Self::BasicUi => &["/qb+"],
            Self::Quiet => &["/quiet"],
            Self::Passive => &["/passive"],
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdaterWindowsConfig {
    #[serde(default, alias = "install-mode")]
    pub install_mode: WindowsUpdateInstallMode,
}

/// A URL to an updater server.
///
/// The URL must use the `https` scheme on production.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct UpdaterEndpoint(pub Url);

impl std::fmt::Display for UpdaterEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<'de> Deserialize<'de> for UpdaterEndpoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let url = Url::deserialize(deserializer)?;
        #[cfg(all(not(debug_assertions), not(feature = "schema")))]
        {
            if url.scheme() != "https" {
                return Err(serde::de::Error::custom(
                    "The configured updater endpoint must use the `https` protocol.",
                ));
            }
        }
        Ok(Self(url))
    }
}
