use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub scope: FsScope,
}

/// Protocol scope definition.
/// It is a list of glob patterns that restrict the API access from the webview.
///
/// Each pattern can start with a variable that resolves to a system base directory.
/// The variables are: `$AUDIO`, `$CACHE`, `$CONFIG`, `$DATA`, `$LOCALDATA`, `$DESKTOP`,
/// `$DOCUMENT`, `$DOWNLOAD`, `$EXE`, `$FONT`, `$HOME`, `$PICTURE`, `$PUBLIC`, `$RUNTIME`,
/// `$TEMPLATE`, `$VIDEO`, `$RESOURCE`, `$APP`, `$LOG`, `$TEMP`, `$APPCONFIG`, `$APPDATA`,
/// `$APPLOCALDATA`, `$APPCACHE`, `$APPLOG`.
#[derive(Debug, PartialEq, Eq, Clone, Deserialize)]
#[serde(untagged)]
pub enum FsScope {
    /// A list of paths that are allowed by this scope.
    AllowedPaths(Vec<PathBuf>),
    /// A complete scope configuration.
    Scope {
        /// A list of paths that are allowed by this scope.
        #[serde(default)]
        allow: Vec<PathBuf>,
        /// A list of paths that are not allowed by this scope.
        /// This gets precedence over the [`Self::Scope::allow`] list.
        #[serde(default)]
        deny: Vec<PathBuf>,
    },
}

impl Default for FsScope {
    fn default() -> Self {
        Self::AllowedPaths(Vec::new())
    }
}

impl FsScope {
    /// The list of allowed paths.
    pub fn allowed_paths(&self) -> &Vec<PathBuf> {
        match self {
            Self::AllowedPaths(p) => p,
            Self::Scope { allow, .. } => allow,
        }
    }

    /// The list of forbidden paths.
    pub fn forbidden_paths(&self) -> Option<&Vec<PathBuf>> {
        match self {
            Self::AllowedPaths(_) => None,
            Self::Scope { deny, .. } => Some(deny),
        }
    }
}
