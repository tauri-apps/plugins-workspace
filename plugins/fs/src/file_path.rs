// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
    convert::Infallible,
    path::{Path, PathBuf},
    str::FromStr,
};

use serde::Serialize;
use tauri::path::SafePathBuf;

use crate::{Error, Result};

/// Represents either a filesystem path or a URI pointing to a file
/// such as `file://` URIs or Android `content://` URIs.
#[derive(Debug, Serialize, Clone)]
#[serde(untagged)]
pub enum FilePath {
    /// `file://` URIs or Android `content://` URIs.
    Url(url::Url),
    /// Regular [`PathBuf`]
    Path(PathBuf),
}

/// Represents either a safe filesystem path or a URI pointing to a file
/// such as `file://` URIs or Android `content://` URIs.
#[derive(Debug, Clone, Serialize)]
pub enum SafeFilePath {
    /// `file://` URIs or Android `content://` URIs.
    Url(url::Url),
    /// Safe [`PathBuf`], see [`SafePathBuf``].
    Path(SafePathBuf),
}

impl FilePath {
    /// Get a reference to the contaiend [`Path`] if the variant is [`FilePath::Path`].
    ///
    /// Use [`FilePath::into_path`] to try to convert the [`FilePath::Url`] variant as well.
    #[inline]
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Url(_) => None,
            Self::Path(p) => Some(p),
        }
    }

    /// Try to convert into [`PathBuf`] if possible.
    ///
    /// This calls [`Url::to_file_path`](url::Url::to_file_path) if the variant is [`FilePath::Url`],
    /// otherwise returns the contained [PathBuf] as is.
    #[inline]
    pub fn into_path(self) -> Result<PathBuf> {
        match self {
            Self::Url(url) => url
                .to_file_path()
                .map(PathBuf::from)
                .map_err(|_| Error::InvalidPathUrl),
            Self::Path(p) => Ok(p),
        }
    }

    /// Takes the contained [`PathBuf`] if the variant is [`FilePath::Path`],
    /// and when possible, converts Windows UNC paths to regular paths.
    #[inline]
    pub fn simplified(self) -> Self {
        match self {
            Self::Url(url) => Self::Url(url),
            Self::Path(p) => Self::Path(dunce::simplified(&p).to_path_buf()),
        }
    }
}

impl SafeFilePath {
    /// Get a reference to the contaiend [`Path`] if the variant is [`SafeFilePath::Path`].
    ///
    /// Use [`SafeFilePath::into_path`] to try to convert the [`SafeFilePath::Url`] variant as well.
    #[inline]
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Url(_) => None,
            Self::Path(p) => Some(p.as_ref()),
        }
    }

    /// Try to convert into [`PathBuf`] if possible.
    ///
    /// This calls [`Url::to_file_path`](url::Url::to_file_path) if the variant is [`SafeFilePath::Url`],
    /// otherwise returns the contained [PathBuf] as is.
    #[inline]
    pub fn into_path(self) -> Result<PathBuf> {
        match self {
            Self::Url(url) => url
                .to_file_path()
                .map(PathBuf::from)
                .map_err(|_| Error::InvalidPathUrl),
            Self::Path(p) => Ok(p.as_ref().to_owned()),
        }
    }

    /// Takes the contained [`PathBuf`] if the variant is [`SafeFilePath::Path`],
    /// and when possible, converts Windows UNC paths to regular paths.
    #[inline]
    pub fn simplified(self) -> Self {
        match self {
            Self::Url(url) => Self::Url(url),
            Self::Path(p) => {
                // Safe to unwrap since it was a safe file path already
                Self::Path(SafePathBuf::new(dunce::simplified(p.as_ref()).to_path_buf()).unwrap())
            }
        }
    }
}

impl std::fmt::Display for FilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url(u) => u.fmt(f),
            Self::Path(p) => p.display().fmt(f),
        }
    }
}

impl std::fmt::Display for SafeFilePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Url(u) => u.fmt(f),
            Self::Path(p) => p.display().fmt(f),
        }
    }
}

impl<'de> serde::Deserialize<'de> for FilePath {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FilePathVisitor;

        impl<'de> serde::de::Visitor<'de> for FilePathVisitor {
            type Value = FilePath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing an file URL or a path")
            }

            fn visit_str<E>(self, s: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                FilePath::from_str(s).map_err(|e| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(s),
                        &e.to_string().as_str(),
                    )
                })
            }
        }

        deserializer.deserialize_str(FilePathVisitor)
    }
}

impl<'de> serde::Deserialize<'de> for SafeFilePath {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct SafeFilePathVisitor;

        impl<'de> serde::de::Visitor<'de> for SafeFilePathVisitor {
            type Value = SafeFilePath;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing an file URL or a path")
            }

            fn visit_str<E>(self, s: &str) -> std::result::Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                SafeFilePath::from_str(s).map_err(|e| {
                    serde::de::Error::invalid_value(
                        serde::de::Unexpected::Str(s),
                        &e.to_string().as_str(),
                    )
                })
            }
        }

        deserializer.deserialize_str(SafeFilePathVisitor)
    }
}

impl FromStr for FilePath {
    type Err = Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if let Ok(url) = url::Url::from_str(s) {
            if url.scheme().len() != 1 {
                return Ok(Self::Url(url));
            }
        }
        Ok(Self::Path(PathBuf::from(s)))
    }
}

impl FromStr for SafeFilePath {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        if let Ok(url) = url::Url::from_str(s) {
            if url.scheme().len() != 1 {
                return Ok(Self::Url(url));
            }
        }

        SafePathBuf::new(s.into())
            .map(SafeFilePath::Path)
            .map_err(Error::UnsafePathBuf)
    }
}

impl From<PathBuf> for FilePath {
    fn from(value: PathBuf) -> Self {
        Self::Path(value)
    }
}

impl TryFrom<PathBuf> for SafeFilePath {
    type Error = Error;
    fn try_from(value: PathBuf) -> Result<Self> {
        SafePathBuf::new(value)
            .map(SafeFilePath::Path)
            .map_err(Error::UnsafePathBuf)
    }
}

impl From<&Path> for FilePath {
    fn from(value: &Path) -> Self {
        Self::Path(value.to_owned())
    }
}

impl TryFrom<&Path> for SafeFilePath {
    type Error = Error;
    fn try_from(value: &Path) -> Result<Self> {
        SafePathBuf::new(value.to_path_buf())
            .map(SafeFilePath::Path)
            .map_err(Error::UnsafePathBuf)
    }
}

impl From<&PathBuf> for FilePath {
    fn from(value: &PathBuf) -> Self {
        Self::Path(value.to_owned())
    }
}

impl TryFrom<&PathBuf> for SafeFilePath {
    type Error = Error;
    fn try_from(value: &PathBuf) -> Result<Self> {
        SafePathBuf::new(value.to_owned())
            .map(SafeFilePath::Path)
            .map_err(Error::UnsafePathBuf)
    }
}

impl From<url::Url> for FilePath {
    fn from(value: url::Url) -> Self {
        Self::Url(value)
    }
}

impl From<url::Url> for SafeFilePath {
    fn from(value: url::Url) -> Self {
        Self::Url(value)
    }
}

impl TryFrom<FilePath> for PathBuf {
    type Error = Error;
    fn try_from(value: FilePath) -> Result<Self> {
        value.into_path()
    }
}

impl TryFrom<SafeFilePath> for PathBuf {
    type Error = Error;
    fn try_from(value: SafeFilePath) -> Result<Self> {
        value.into_path()
    }
}

impl From<SafeFilePath> for FilePath {
    fn from(value: SafeFilePath) -> Self {
        match value {
            SafeFilePath::Url(url) => FilePath::Url(url),
            SafeFilePath::Path(p) => FilePath::Path(p.as_ref().to_owned()),
        }
    }
}

impl TryFrom<FilePath> for SafeFilePath {
    type Error = Error;

    fn try_from(value: FilePath) -> Result<Self> {
        match value {
            FilePath::Url(url) => Ok(SafeFilePath::Url(url)),
            FilePath::Path(p) => SafePathBuf::new(p)
                .map(SafeFilePath::Path)
                .map_err(Error::UnsafePathBuf),
        }
    }
}
