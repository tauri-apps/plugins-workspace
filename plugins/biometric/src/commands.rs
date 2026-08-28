// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Desktop commands.
//!
//! On mobile these two names are served by the native plugin; on macOS they are
//! ordinary commands so that `plugin:biometric|status` and
//! `plugin:biometric|authenticate` mean the same thing on every platform.

use tauri::{command, AppHandle, Runtime};

use crate::models::{AuthOptions, Status};

#[command]
pub(crate) async fn status<R: Runtime>(_app: AppHandle<R>) -> Status {
    // Cheap and non-blocking; no need to leave the runtime for it.
    crate::macos::status()
}

/// The JS API sends `{ reason, ...options }` — the options are flattened into
/// the payload rather than nested — so the fields are taken individually here.
/// Matching that shape is what lets one frontend target mobile and macOS
/// without branching.
#[command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn authenticate<R: Runtime>(
    _app: AppHandle<R>,
    reason: String,
    allow_device_credential: Option<bool>,
    fallback_title: Option<String>,
    cancel_title: Option<String>,
    title: Option<String>,
    subtitle: Option<String>,
    confirmation_required: Option<bool>,
) -> crate::Result<()> {
    let options = AuthOptions {
        allow_device_credential: allow_device_credential.unwrap_or(false),
        fallback_title,
        cancel_title,
        title,
        subtitle,
        confirmation_required,
    };
    // The prompt blocks until the user answers, so it cannot run on the async
    // runtime — and it must not run on the main thread either, which is the one
    // presenting the sheet.
    tauri::async_runtime::spawn_blocking(move || crate::macos::authenticate(reason, options))
        .await
        .map_err(|e| crate::Error::Biometric(e.to_string()))?
}
