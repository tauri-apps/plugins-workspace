// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::{models::*, FilePath, OpenOptions};

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "com.plugin.fs";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_fs);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Fs<R>> {
    #[cfg(target_os = "android")]
    let handle = api
        .register_android_plugin(PLUGIN_IDENTIFIER, "FsPlugin")
        .unwrap();
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_android - intent - send)?;
    Ok(Fs(handle))
}

/// Access to the android-intent-send APIs.
pub struct Fs<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Fs<R> {
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

    #[cfg(target_os = "android")]
    fn resolve_content_uri(
        &self,
        uri: impl Into<String>,
        mode: impl Into<String>,
    ) -> crate::Result<std::fs::File> {
        #[cfg(target_os = "android")]
        {
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
                todo!()
            }
        }
    }
}
