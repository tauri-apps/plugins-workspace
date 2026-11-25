// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{AppHandle, Runtime, plugin::PluginApi};

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
            Err(crate::Error::Unavailable {
                message: "Biometric authentication is not supported on this platform".into(),
                code: Some("biometryNotAvailable".into()),
            })
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use block2::RcBlock;
    use objc2::runtime::Bool;
    use objc2_foundation::{NSError, NSString};
    use objc2_local_authentication::{LABiometryType, LAContext, LAError, LAPolicy};

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
                // Same fallback as the iOS plugin for codes outside the documented set.
                let code = map_la_error_code(error.code()).unwrap_or("biometryNotAvailable");

                Ok(Status {
                    is_available: false,
                    biometry_type: convert_biometry_type(biometry_type),
                    error: Some(desc),
                    error_code: Some(code.into()),
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
        if let Err(error) = can_evaluate
            && !options.allow_device_credential
        {
            let code = map_la_error_code(error.code()).unwrap_or("biometryNotAvailable");
            return Err(crate::Error::Unavailable {
                message: error.localizedDescription().to_string(),
                code: Some(code.into()),
            });
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
                let err = if !error.is_null() {
                    let e = unsafe { &*error };
                    crate::Error::AuthenticationFailed {
                        message: e.localizedDescription().to_string(),
                        code: map_la_error_code(e.code()).map(Into::into),
                    }
                } else {
                    crate::Error::AuthenticationFailed {
                        message: "Unknown error".into(),
                        code: Some("authenticationFailed".into()),
                    }
                };
                tx.send(Err(err)).ok();
            }
        });

        unsafe {
            context.evaluatePolicy_localizedReason_reply(policy, &reason, &block);
        }

        rx.recv().map_err(|_| crate::Error::AuthenticationFailed {
            message: "Channel closed".into(),
            code: None,
        })?
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

    /// Maps an `LAError` code to the identifiers documented in the JS API, matching the iOS plugin.
    fn map_la_error_code(code: isize) -> Option<&'static str> {
        let code = match LAError(code) {
            LAError::AuthenticationFailed => "authenticationFailed",
            LAError::UserCancel => "userCancel",
            LAError::UserFallback => "userFallback",
            LAError::SystemCancel => "systemCancel",
            LAError::PasscodeNotSet => "passcodeNotSet",
            LAError::BiometryNotAvailable => "biometryNotAvailable",
            LAError::BiometryNotEnrolled => "biometryNotEnrolled",
            LAError::BiometryLockout => "biometryLockout",
            LAError::AppCancel => "appCancel",
            LAError::InvalidContext => "invalidContext",
            LAError::NotInteractive => "notInteractive",
            _ => return None,
        };
        Some(code)
    }
}
