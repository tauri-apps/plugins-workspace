// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{AppHandle, Runtime, State, command, plugin::PermissionState};

use crate::{Notification, NotificationData, Result};

#[command]
pub(crate) async fn is_permission_granted<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notification<R>>,
) -> Result<Option<bool>> {
    let state = notification.permission_state()?;
    match state {
        PermissionState::Granted => Ok(Some(true)),
        PermissionState::Denied => Ok(Some(false)),
        PermissionState::Prompt | PermissionState::PromptWithRationale => Ok(None),
    }
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
    options: NotificationData,
) -> Result<()> {
    let mut builder = notification.builder();
    builder.data = options;
    builder.show()
}

// The commands below are implemented by the Kotlin and Swift plugins on mobile.

#[cfg(desktop)]
#[command]
pub(crate) async fn register_action_types<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notification<R>>,
    types: Vec<crate::ActionType>,
) -> Result<()> {
    notification.register_action_types(types)
}

/// A notification of `remove_active`, as the JavaScript API sends it.
#[cfg(desktop)]
#[derive(serde::Deserialize)]
pub(crate) struct ActiveId {
    id: i32,
}

#[cfg(desktop)]
#[command]
pub(crate) async fn remove_active<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notification<R>>,
    notifications: Option<Vec<ActiveId>>,
) -> Result<()> {
    match notifications {
        Some(notifications) => {
            notification.remove_active(notifications.into_iter().map(|n| n.id).collect())
        }
        None => notification.remove_all_active(),
    }
}

#[cfg(desktop)]
#[command]
pub(crate) fn register_listener<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notification<R>>,
    event: String,
    handler: tauri::ipc::Channel<serde_json::Value>,
) {
    notification.register_listener(event, handler);
}

#[cfg(desktop)]
#[command]
pub(crate) fn remove_listener<R: Runtime>(
    _app: AppHandle<R>,
    notification: State<'_, Notification<R>>,
    event: String,
    channel_id: u32,
) {
    notification.remove_listener(&event, channel_id);
}
