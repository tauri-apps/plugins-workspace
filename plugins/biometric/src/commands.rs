// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{AppHandle, Runtime, State, command};

use crate::{Biometric, BiometricExt, Result, models::*};

#[command]
pub(crate) async fn status<R: Runtime>(
    _app: AppHandle<R>,
    biometric: State<'_, Biometric<R>>,
) -> Result<Status> {
    biometric.status()
}

#[command]
#[allow(clippy::too_many_arguments)]
pub(crate) async fn authenticate<R: Runtime>(
    app: AppHandle<R>,
    reason: String,
    allow_device_credential: Option<bool>,
    cancel_title: Option<String>,
    fallback_title: Option<String>,
    title: Option<String>,
    subtitle: Option<String>,
    confirmation_required: Option<bool>,
) -> Result<()> {
    let options = AuthOptions {
        allow_device_credential: allow_device_credential.unwrap_or(false),
        cancel_title,
        fallback_title,
        title,
        subtitle,
        confirmation_required,
        ..Default::default()
    };
    // The prompt blocks until the user answers, so keep it off the async runtime workers.
    tauri::async_runtime::spawn_blocking(move || app.biometric().authenticate(reason, options))
        .await
        .map_err(|e| crate::Error::AuthenticationFailed {
            message: e.to_string(),
            code: None,
        })?
}
