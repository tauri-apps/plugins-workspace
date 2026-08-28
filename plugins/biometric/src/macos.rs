// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Touch ID on macOS, through LocalAuthentication.
//!
//! The mobile plugins hand these calls to a native plugin; macOS has no such
//! plugin, so the framework is driven directly here and the same two commands
//! are served over the desktop invoke handler. The API a caller sees —
//! `status`, `authenticate`, the `Status` and `AuthOptions` shapes — is the one
//! the mobile platforms already expose, so a frontend written against either
//! works unchanged.

use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_foundation::{NSError, NSString};
use objc2_local_authentication::{LABiometryType, LAContext, LAPolicy};

use crate::models::{AuthOptions, BiometryType, Status};

/// A user is not going to sit in front of a Touch ID prompt for longer than
/// this, and the reply block must not be able to strand a caller forever.
const AUTH_TIMEOUT: Duration = Duration::from_secs(120);

/// Which policy a request maps to.
///
/// `allow_device_credential` is the same promise the mobile platforms make:
/// fall back to the device passcode when biometry cannot be used. On macOS that
/// is the login password, which is what `DeviceOwnerAuthentication` evaluates.
fn policy(options: &AuthOptions) -> LAPolicy {
    if options.allow_device_credential {
        LAPolicy::DeviceOwnerAuthentication
    } else {
        LAPolicy::DeviceOwnerAuthenticationWithBiometrics
    }
}

fn biometry_type(context: &LAContext) -> BiometryType {
    // Only meaningful after a policy has been evaluated or pre-flighted; the
    // caller below always pre-flights first.
    match unsafe { context.biometryType() } {
        LABiometryType::TouchID => BiometryType::TouchID,
        LABiometryType::FaceID => BiometryType::FaceID,
        _ => BiometryType::None,
    }
}

/// Map an `LAError` code to the string identifiers the JS API documents, so a
/// frontend can branch on the same values it would on iOS.
fn error_code(error: &NSError) -> Option<String> {
    // Values from LAError.h; kept as literals rather than pulling in the enum,
    // which is spelled differently across the objc2 crate versions.
    let code = match error.code() {
        -1 => "authenticationFailed",
        -2 => "userCancel",
        -3 => "userFallback",
        -4 => "systemCancel",
        -5 => "passcodeNotSet",
        -8 => "biometryLockout",
        -6 => "biometryNotAvailable",
        -7 => "biometryNotEnrolled",
        -9 => "appCancel",
        -10 => "invalidContext",
        -1004 => "notInteractive",
        _ => return None,
    };
    Some(code.to_string())
}

fn message(error: &NSError) -> String {
    error.localizedDescription().to_string()
}

/// Whether biometric authentication can be used right now.
pub fn status() -> Status {
    let context = unsafe { LAContext::new() };
    match unsafe {
        context.canEvaluatePolicy_error(LAPolicy::DeviceOwnerAuthenticationWithBiometrics)
    } {
        Ok(()) => Status {
            is_available: true,
            biometry_type: biometry_type(&context),
            error: None,
            error_code: None,
        },
        Err(error) => Status {
            is_available: false,
            // Report the hardware even when it is unusable: "you have Touch ID
            // but nothing is enrolled" is a different message from "this Mac
            // has no Touch ID", and only the type distinguishes them.
            biometry_type: biometry_type(&context),
            error: Some(message(&error)),
            error_code: error_code(&error),
        },
    }
}

/// Prompt for Touch ID, blocking until the user answers.
///
/// `evaluatePolicy` is asynchronous and answers on an arbitrary queue, so the
/// reply is funnelled through a channel. Callers must not run this on the main
/// thread: the system presents the sheet itself, and blocking the main thread
/// while it does would deadlock the app.
pub fn authenticate(reason: String, options: AuthOptions) -> crate::Result<()> {
    let context = unsafe { LAContext::new() };
    let policy = policy(&options);

    // Pre-flight, so an unusable policy produces the real reason rather than a
    // bare "authentication failed" from the prompt.
    if let Err(error) = unsafe { context.canEvaluatePolicy_error(policy) } {
        return Err(crate::Error::Biometric(message(&error)));
    }

    if let Some(title) = options.cancel_title.as_deref() {
        unsafe { context.setLocalizedCancelTitle(Some(&NSString::from_str(title))) };
    }
    if let Some(title) = options.fallback_title.as_deref() {
        unsafe { context.setLocalizedFallbackTitle(Some(&NSString::from_str(title))) };
    }

    let (tx, rx) = mpsc::channel::<Result<(), String>>();
    let reply = RcBlock::new(move |success: Bool, error: *mut NSError| {
        let outcome = if success.as_bool() {
            Ok(())
        } else {
            // SAFETY: LocalAuthentication passes an autoreleased error whenever
            // it reports failure; null is only possible alongside success.
            let described = unsafe { error.as_ref() }
                .map(message)
                .unwrap_or_else(|| "authentication failed".to_string());
            Err(described)
        };
        // The receiver is gone only if we timed out; nothing to do about it.
        let _ = tx.send(outcome);
    });

    let reason = NSString::from_str(&reason);
    unsafe { context.evaluatePolicy_localizedReason_reply(policy, &reason, &reply) };

    // The context has to outlive the evaluation — dropping it cancels the
    // prompt — which the blocking receive below guarantees.
    let outcome = rx.recv_timeout(AUTH_TIMEOUT);
    drop::<Retained<LAContext>>(context);

    match outcome {
        Ok(Ok(())) => Ok(()),
        Ok(Err(message)) => Err(crate::Error::Biometric(message)),
        Err(_) => Err(crate::Error::Biometric(
            "the authentication prompt timed out".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_answers_without_prompting() {
        // Pre-flight only: this must never present UI, so it is safe in a test
        // run and safe to call on every app launch.
        let status = status();
        if status.is_available {
            assert!(
                matches!(status.biometry_type, BiometryType::TouchID | BiometryType::FaceID),
                "available biometry must name its type"
            );
            assert!(status.error.is_none());
        } else {
            // Unavailable is a legitimate answer on a Mac without Touch ID, but
            // it has to say why rather than failing silently.
            assert!(
                status.error.is_some(),
                "unavailable biometry must carry a reason"
            );
        }
    }

    #[test]
    fn device_credential_selects_the_broader_policy() {
        let with = AuthOptions {
            allow_device_credential: true,
            ..Default::default()
        };
        let without = AuthOptions::default();
        assert_eq!(policy(&with), LAPolicy::DeviceOwnerAuthentication);
        assert_eq!(
            policy(&without),
            LAPolicy::DeviceOwnerAuthenticationWithBiometrics
        );
    }
}
