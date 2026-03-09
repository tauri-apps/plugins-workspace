// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::{FilePath, OpenOptions};

pub struct Fs<R: Runtime> {
    _phantom: std::marker::PhantomData<fn() -> R>,
}

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Fs<R>> {
    Ok(Fs {
        _phantom: std::marker::PhantomData,
    })
}

impl<R: Runtime> Fs<R> {
    /// Open a file.
    ///
    /// # Platform-specific
    ///
    /// - **iOS**: This method will automatically start accessing a security-scoped resource if the path is a file URL.
    ///   You must call [`Self::stop_accessing_security_scoped_resource`] when you're done accessing the file.
    pub fn open<P: Into<FilePath>>(
        &self,
        path: P,
        opts: OpenOptions,
    ) -> std::io::Result<std::fs::File> {
        use objc2_foundation::{NSString, NSURL};

        match path.into() {
            FilePath::Url(url) if url.scheme() == "file" => {
                // Handle security-scoped URLs on iOS
                let url_string = url.as_str();
                let url_nsstring = NSString::from_str(url_string);

                // Create NSURL from the URL string
                // URLWithString may return None for invalid URLs, but file:// URLs should be valid
                let ns_url = unsafe { NSURL::URLWithString(&url_nsstring) };
                if let Some(ns_url) = ns_url {
                    // Start accessing the security-scoped resource
                    // This is required for files outside the app's sandbox (e.g., from file picker)
                    // Note: We don't call stopAccessingSecurityScopedResource here because
                    // the file handle needs to remain accessible while the File is in use.
                    // The access will be automatically stopped when the app is backgrounded or terminated.
                    unsafe {
                        let success = ns_url.startAccessingSecurityScopedResource();
                        if success {
                            log::debug!(
                                "Started accessing security-scoped resource for URL: {}",
                                url_string
                            );
                        } else {
                            log::warn!(
                                "Failed to start accessing security-scoped resource for URL: {}",
                                url_string
                            );
                        }
                    }
                } else {
                    log::debug!("Failed to create NSURL from URL: {}, ignoring security-scoped resource access request", url_string);
                }

                // Convert URL to path and open the file
                let path = url.to_file_path().map_err(|_| {
                    std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid file URL")
                })?;
                std::fs::OpenOptions::from(opts).open(path)
            }
            FilePath::Url(_) => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "cannot use a non-file URL to load files on iOS",
            )),
            FilePath::Path(p) => {
                // Regular path, no security-scoped resource handling needed
                std::fs::OpenOptions::from(opts).open(p)
            }
        }
    }

    /// Stops accessing a security-scoped resource for the given file path or URL.
    /// This should be called when you're done accessing a file that was opened
    /// using a security-scoped URL (e.g., from a file picker).
    ///
    /// # Arguments
    ///
    /// * `path` - A file path or URL that was previously accessed via `startAccessingSecurityScopedResource`
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` if successful, or an error if the path/URL is invalid or not a file URL.
    pub fn stop_accessing_security_scoped_resource<P: Into<FilePath>>(
        &self,
        path: P,
    ) -> crate::Result<()> {
        use objc2_foundation::{NSString, NSURL};

        let file_path = path.into();
        let url_string = match file_path {
            FilePath::Url(url) => {
                if url.scheme() != "file" {
                    return Err(crate::Error::InvalidPathUrl);
                }
                url.as_str().to_string()
            }
            FilePath::Path(p) => {
                // Convert path to file URL
                url::Url::from_file_path(&p)
                    .map_err(|_| crate::Error::InvalidPathUrl)?
                    .as_str()
                    .to_string()
            }
        };

        let url_nsstring = NSString::from_str(&url_string);
        let ns_url = unsafe { NSURL::URLWithString(&url_nsstring) };
        if let Some(ns_url) = ns_url {
            // Stop accessing the security-scoped resource
            unsafe {
                ns_url.stopAccessingSecurityScopedResource();
            }
        } else {
            return Err(crate::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "failed to create NSURL from URL",
            )));
        }

        Ok(())
    }
}
