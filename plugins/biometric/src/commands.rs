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

#[command]
pub(crate) async fn authenticate<R: Runtime>(
    _app: AppHandle<R>,
    reason: String,
    #[allow(clippy::used_underscore_binding)] options: AuthOptions,
) -> crate::Result<()> {
    // The prompt blocks until the user answers, so it cannot run on the async
    // runtime — and it must not run on the main thread either, which is the one
    // presenting the sheet.
    tauri::async_runtime::spawn_blocking(move || crate::macos::authenticate(reason, options))
        .await
        .map_err(|e| crate::Error::Biometric(e.to_string()))?
}
