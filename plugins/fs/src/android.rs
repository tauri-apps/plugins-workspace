// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::{models::*, FilePath, OpenOptions};

const PLUGIN_IDENTIFIER: &str = "com.plugin.fs";

pub struct Fs<R: Runtime>(tauri::plugin::PluginHandle<R>);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Fs<R>> {
    let handle = api
        .register_android_plugin(PLUGIN_IDENTIFIER, "FsPlugin")
        .unwrap();
    Ok(Fs(handle))
}

impl<R: Runtime> Fs<R> {
    /// Open a file.
    ///
    /// # Platform-specific
    ///
    /// - **iOS**: This method will automatically start accessing a security-scoped resource if the path is a file URL.
    ///   You must call `stop_accessing_security_scoped_resource` when you're done accessing the file.
    pub fn open<P: Into<FilePath>>(
        &self,
        path: P,
        opts: OpenOptions,
    ) -> std::io::Result<std::fs::File> {
        match path.into() {
            FilePath::Url(u) => self
                .resolve_content_uri(u.to_string(), opts.android_mode())
                .map_err(|e| {
                    std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("failed to open file: {e}"),
                    )
                }),
            FilePath::Path(p) => {
                // tauri::utils::platform::resources_dir() returns a PathBuf with the Android asset URI prefix
                // we must resolve that file with the Android API
                if p.strip_prefix(tauri::utils::platform::ANDROID_ASSET_PROTOCOL_URI_PREFIX)
                    .is_ok()
                {
                    self.resolve_content_uri(p.to_string_lossy(), opts.android_mode())
                        .map_err(|e| {
                            std::io::Error::new(
                                std::io::ErrorKind::Other,
                                format!("failed to open file: {e}"),
                            )
                        })
                } else {
                    std::fs::OpenOptions::from(opts).open(p)
                }
            }
        }
    }

    fn resolve_content_uri(
        &self,
        uri: impl Into<String>,
        mode: impl Into<String>,
    ) -> crate::Result<std::fs::File> {
        let result = self.0.run_mobile_plugin::<GetFileDescriptorResponse>(
            "getFileDescriptor",
            GetFileDescriptorPayload {
                uri: uri.into(),
                mode: mode.into(),
            },
        )?;
        if let Some(fd) = result.fd {
            Ok(unsafe {
                use std::os::fd::FromRawFd;
                std::fs::File::from_raw_fd(fd)
            })
        } else {
            unimplemented!()
        }
    }
}
