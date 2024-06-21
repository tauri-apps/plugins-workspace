// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, ipc::Channel, AppHandle};

use crate::{GeolocationExt, PermissionStatus, PermissionType, Position, PositionOptions, Result};

#[command]
#[specta::specta]
pub(crate) async fn get_current_position(
    app: AppHandle,
    options: Option<PositionOptions>,
) -> Result<Position> {
    app.geolocation().get_current_position(options)
}

#[command]
#[specta::specta]
pub(crate) async fn watch_position(
    app: AppHandle,
    options: PositionOptions,
    channel: Channel,
) -> Result<()> {
    app.geolocation().watch_position_inner(options, channel)
}

#[command]
#[specta::specta]
pub(crate) async fn clear_watch(app: AppHandle, channel_id: u32) -> Result<()> {
    app.geolocation().clear_watch(channel_id)
}

#[command]
#[specta::specta]
pub(crate) async fn check_permissions(app: AppHandle) -> Result<PermissionStatus> {
    app.geolocation().check_permissions()
}

#[command]
#[specta::specta]
pub(crate) async fn request_permissions(
    app: AppHandle,
    permissions: Option<Vec<PermissionType>>,
) -> Result<PermissionStatus> {
    app.geolocation().request_permissions(permissions)
}
