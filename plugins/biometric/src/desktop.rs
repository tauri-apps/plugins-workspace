// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Biometric<R>> {
    Ok(Biometric(app.clone()))
}

/// Access to the biometric APIs.
pub struct Biometric<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Biometric<R> {
    pub fn status(&self) -> crate::Result<Status> {
        #[cfg(target_os = "macos")]
        {
            macos::status()
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(Status {
                is_available: false,
                biometry_type: BiometryType::None,
                error: Some("Biometric authentication is not supported on this platform".into()),
                error_code: Some("biometryNotAvailable".into()),
            })
        }
    }

    pub fn authenticate(&self, reason: String, options: AuthOptions) -> crate::Result<()> {
        #[cfg(target_os = "macos")]
        {
            macos::authenticate(reason, options)
        }
        #[cfg(not(target_os = "macos"))]
        {
            let _ = (reason, options);
            Err(crate::Error::Unavailable(
                "Biometric authentication is not supported on this platform".into(),
            ))
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_foundation::{NSError, NSString};
    use objc2_local_authentication::{LABiometryType, LAContext, LAPolicy};

    pub fn status() -> crate::Result<Status> {
        let context = unsafe { LAContext::new() };
        let result = unsafe {
            context.canEvaluatePolicy_error(LAPolicy::DeviceOwnerAuthenticationWithBiometrics)
        };
        let biometry_type = unsafe { context.biometryType() };

        match result {
            Ok(()) => Ok(Status {
                is_available: true,
                biometry_type: convert_biometry_type(biometry_type),
                error: None,
                error_code: None,
            }),
            Err(error) => {
                let desc = error.localizedDescription().to_string();
                let code = map_la_error_code(error.code());

                Ok(Status {
                    is_available: false,
                    biometry_type: convert_biometry_type(biometry_type),
                    error: Some(desc),
                    error_code: Some(code),
                })
            }
        }
    }

    pub fn authenticate(reason: String, options: AuthOptions) -> crate::Result<()> {
        let context = unsafe { LAContext::new() };

        // Pre-check: if biometry is unavailable and device credential fallback is disabled,
        // return early with the error.
        let can_evaluate = unsafe {
            context.canEvaluatePolicy_error(LAPolicy::DeviceOwnerAuthenticationWithBiometrics)
        };
        if let Err(error) = can_evaluate {
            if !options.allow_device_credential {
                let desc = error.localizedDescription().to_string();
                let code = map_la_error_code(error.code());
                return Err(crate::Error::Unavailable(format!("[{code}] {desc}")));
            }
        }

        // Set localized titles
        if let Some(ref fallback_title) = options.fallback_title {
            let title = NSString::from_str(fallback_title);
            unsafe { context.setLocalizedFallbackTitle(Some(&title)) };
        }
        if options.allow_device_credential && matches!(options.fallback_title.as_deref(), Some(""))
        {
            unsafe { context.setLocalizedFallbackTitle(None) };
        }
        if let Some(ref cancel_title) = options.cancel_title {
            let title = NSString::from_str(cancel_title);
            unsafe { context.setLocalizedCancelTitle(Some(&title)) };
        }

        // Disable authentication reuse
        unsafe { context.setTouchIDAuthenticationAllowableReuseDuration(0.0) };

        let reason = NSString::from_str(&reason);
        let policy = if options.allow_device_credential {
            LAPolicy::DeviceOwnerAuthentication
        } else {
            LAPolicy::DeviceOwnerAuthenticationWithBiometrics
        };

        let (tx, rx) = std::sync::mpsc::channel();
        let block = RcBlock::new(move |success: Bool, error: *mut NSError| {
            if success.as_bool() {
                tx.send(Ok(())).ok();
            } else {
                let err_msg = if !error.is_null() {
                    let e = unsafe { &*error };
                    let desc = e.localizedDescription().to_string();
                    let code = map_la_error_code(e.code());
                    format!("[{code}] {desc}")
                } else {
                    "Authentication failed".to_string()
                };
                tx.send(Err(crate::Error::AuthenticationFailed(err_msg)))
                    .ok();
            }
        });

        unsafe {
            context.evaluatePolicy_localizedReason_reply(policy, &reason, &block);
        }

        rx.recv()
            .map_err(|_| crate::Error::AuthenticationFailed("Channel closed".into()))?
    }

    fn convert_biometry_type(biometry_type: LABiometryType) -> BiometryType {
        if biometry_type == LABiometryType::TouchID {
            BiometryType::TouchID
        } else if biometry_type == LABiometryType::FaceID {
            BiometryType::FaceID
        } else {
            BiometryType::None
        }
    }

    fn map_la_error_code(code: isize) -> String {
        match code {
            -1 => "authenticationFailed",
            -2 => "userCancel",
            -3 => "userFallback",
            -4 => "systemCancel",
            -5 => "passcodeNotSet",
            -6 => "appCancel",
            -7 => "biometryNotAvailable",
            -8 => "biometryNotEnrolled",
            -9 => "biometryLockout",
            -10 => "invalidContext",
            -1004 => "notInteractive",
            _ => "unknown",
        }
        .to_string()
    }
}
