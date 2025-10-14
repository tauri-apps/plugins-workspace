// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use keyring_core::{set_default_store, Entry};
use tauri::{
    plugin::{Builder, TauriPlugin},
    AppHandle, Manager, Runtime,
};

mod commands;
mod error;

pub use error::{Error, Result};

// TODO: Consider using a worker thread to handle caveats mentioned by keyring-rs

/// Extensions to [`tauri::App`], [`tauri::AppHandle`], [`tauri::WebviewWindow`], [`tauri::Webview`] and [`tauri::Window`] to access the secure-storage APIs.
pub trait SecureStorageExt<R: Runtime> {
    fn secure_storage(&self) -> &SecureStorage<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SecureStorageExt<R> for T {
    fn secure_storage(&self) -> &SecureStorage<R> {
        self.state::<SecureStorage<R>>().inner()
    }
}

/// Initializes the plugin.
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("secure-storage")
        .invoke_handler(tauri::generate_handler![
            commands::set_string,
            commands::get_string,
            commands::set_binary,
            commands::get_binary
        ])
        .setup(|app, _api| {
            #[cfg(target_os = "android")]
            set_default_store(android_native_keyring_store::AndroidStore::from_ndk_context()?);

            // TODO: (maybe) config to change used keychain.
            #[cfg(all(target_os = "android", feature = "apple-keychain"))]
            set_default_store(apple_native_keyring_store::keychain::Store::new()?);

            // TODO: config. most notably icloud sync and biometrics
            #[cfg(all(target_os = "android", feature = "apple-protected"))]
            set_default_store(apple_native_keyring_store::protected::Store::new()?);

            #[cfg(windows)]
            set_default_store(windows_native_keyring_store::Store::new()?);

            #[cfg(target_os = "linux")]
            set_default_store(zbus_secret_service_keyring_store::Store::new()?);

            app.manage(SecureStorage(app.clone()));
            Ok(())
        })
        .build()
}

/// Access to the secure-storage APIs.
pub struct SecureStorage<R: Runtime>(AppHandle<R>);

impl<R: Runtime> SecureStorage<R> {
    pub fn set_string(&self, key: &str, value: &str) -> Result<()> {
        Ok(Entry::new(&self.0.config().identifier, key)?.set_password(value)?)
    }

    pub fn get_string(&self, key: &str) -> Result<String> {
        Ok(Entry::new(&self.0.config().identifier, key)?.get_password()?)
    }

    pub fn set_binary(&self, key: &str, value: &[u8]) -> Result<()> {
        Ok(Entry::new(&self.0.config().identifier, key)?.set_secret(value)?)
    }

    pub fn get_binary(&self, key: &str) -> Result<Vec<u8>> {
        Ok(Entry::new(&self.0.config().identifier, key)?.get_secret()?)
    }
}
