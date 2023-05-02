// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use tauri::{command, AppHandle, Runtime, State};

use crate::{Notification, PermissionState, Result};

/// The options for the notification API.
#[derive(Debug, Clone, Deserialize)]
pub struct NotificationOptions {
    /// The notification title.
    pub title: String,
    /// The notification body.
    pub body: Option<String>,
    /// The notification icon.
    pub icon: Option<String>,
}

#[command]
pub(crate) async fn is_permission_granted<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notification<R>>,
) -> Result<bool> {
    notification
        .permission_state()
        .map(|s| s == PermissionState::Granted)
}

#[command]
pub(crate) async fn request_permission<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notification<R>>,
) -> Result<PermissionState> {
    notification.request_permission()
}

#[command]
pub(crate) async fn notify<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notification<R>>,
    options: NotificationOptions,
) -> Result<()> {
    let mut builder = notification.builder().title(options.title);
    if let Some(body) = options.body {
        builder = builder.body(body);
    }
    if let Some(icon) = options.icon {
        builder = builder.icon(icon);
    }

    builder.show()
}
