use std::path::PathBuf;

use serde::{Serialize, Serializer};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("forbidden path: {0}")]
    PathForbidden(PathBuf),
    #[error("failed to resolve path: {0}")]
    CannotResolvePath(tauri::path::Error),
    /// Invalid glob pattern.
    #[error("invalid glob pattern: {0}")]
    GlobPattern(#[from] glob::PatternError),
    /// Watcher error.
    #[cfg(feature = "watch")]
    #[error(transparent)]
    Watch(#[from] notify::Error),
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}
