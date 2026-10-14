// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Serialize, de::DeserializeOwned};
use tauri::{
    AppHandle, Runtime,
    plugin::{PluginApi, PluginHandle},
};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.biometric";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_biometric);

pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Biometric<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "BiometricPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_biometric)?;
    Ok(Biometric(handle))
}

/// Access to the biometric APIs.
pub struct Biometric<R: Runtime>(PluginHandle<R>);

#[derive(Serialize)]
struct AuthenticatePayload {
    reason: String,
    #[serde(flatten)]
    options: AuthOptions,
}

impl<R: Runtime> Biometric<R> {
    /// Checks the device's availability and type of biometric authentication, as reported by the
    /// operating system. Errors if the underlying mobile plugin invocation fails.
    pub fn status(&self) -> crate::Result<Status> {
        self.0.run_mobile_plugin("status", ()).map_err(Into::into)
    }

    /// Prompts the user for biometric authentication using the system UI (Android
    /// `BiometricPrompt` or iOS `LocalAuthentication`), showing `reason` as the purpose of the
    /// request. Resolves once the user is authenticated and errors if authentication fails, is
    /// canceled, or the underlying mobile plugin invocation fails.
    pub fn authenticate(&self, reason: String, options: AuthOptions) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("authenticate", AuthenticatePayload { reason, options })
            .map_err(Into::into)
    }

    /// Prompts the user for biometric authentication and, on success, encrypts
    /// [`AuthOptions::data_to_encrypt`] or decrypts [`AuthOptions::data_to_decrypt`] using a key
    /// protected by the device's strong biometrics. **Android only.**
    pub fn biometric_cipher(
        &self,
        reason: String,
        options: AuthOptions,
    ) -> crate::Result<CipherResult> {
        self.0
            .run_mobile_plugin("biometricCipher", AuthenticatePayload { reason, options })
            .map_err(Into::into)
    }
}
