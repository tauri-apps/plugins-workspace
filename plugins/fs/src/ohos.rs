// Copyright 2019-2026 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{FilePath, OpenOptions};
use serde::de::DeserializeOwned;
use std::{fs::File, os::fd::FromRawFd};
use tauri::{
    AppHandle, Runtime,
    plugin::{PluginApi, PluginHandle},
};

pub struct Fs<R: Runtime>(PluginHandle<R>);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Fs<R>> {
    Ok(Fs(api.register_ohos_plugin()?))
}

impl<R: Runtime> Fs<R> {
    pub fn open<P: Into<FilePath>>(&self, path: P, options: OpenOptions) -> std::io::Result<File> {
        match path.into() {
            FilePath::Path(path) => std::fs::OpenOptions::from(options).open(path),
            FilePath::Url(uri) => {
                if options.mode.is_some() || options.custom_flags.is_some() {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Unsupported,
                        "Unix permissions and custom flags require a filesystem path, not a picker URI",
                    ));
                }
                // ArkTS accepts only URIs granted by this Ability's native picker.
                // Duplicate the descriptor before releasing its ArkTS File owner.
                let fd: i32 = self
                    .0
                    .run_mobile_plugin(
                        "openFile",
                        serde_json::json!({
                            "uri": uri.as_str(), "read": options.read, "write": options.write,
                            "append": options.append, "truncate": options.truncate,
                            "create": options.create, "createNew": options.create_new
                        }),
                    )
                    .map_err(std::io::Error::other)?;
                let owned = unsafe { libc::dup(fd) };
                let file = if owned < 0 {
                    Err(std::io::Error::last_os_error())
                } else {
                    Ok(unsafe { File::from_raw_fd(owned) })
                };
                let released: Result<(), _> = self
                    .0
                    .run_mobile_plugin("closeFile", serde_json::json!({ "fd": fd }));
                released.map_err(std::io::Error::other)?;
                file
            }
        }
    }
}
