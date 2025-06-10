// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use tauri::{AppHandle, Runtime};

use crate::{FilePath, OpenOptions};

pub struct Fs<R: Runtime>(pub(crate) AppHandle<R>);

fn path_or_err<P: Into<FilePath>>(p: P) -> std::io::Result<PathBuf> {
    match p.into() {
        FilePath::Path(p) => Ok(p),
        FilePath::Url(u) if u.scheme() == "file" => u
            .to_file_path()
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid file URL")),
        FilePath::Url(_) => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "cannot use a URL to load files on desktop and iOS",
        )),
    }
}

impl<R: Runtime> Fs<R> {
    pub fn open<P: Into<FilePath>>(
        &self,
        path: P,
        opts: OpenOptions,
    ) -> std::io::Result<std::fs::File> {
        let path = path_or_err(path)?;
        std::fs::OpenOptions::from(opts).open(path)
    }
}
