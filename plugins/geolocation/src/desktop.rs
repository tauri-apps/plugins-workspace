// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{de::DeserializeOwned, Serialize};
use tauri::{
    ipc::Channel,
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Geolocation<R>> {
    Ok(Geolocation(app.clone()))
}

/// Access to the geolocation APIs.
pub struct Geolocation<R: Runtime>(AppHandle<R>);

// TODO: Position instead of Value
impl<R: Runtime> Geolocation<R> {
    pub fn get_current_position(
        &self,
        options: Option<PositionOptions>,
    ) -> crate::Result<Position> {
        Ok(Position::default())
    }

    // TODO: <F: FnMut(Position) + Send + Sync + 'static>
    pub fn watch_position(
        &self,
        options: PositionOptions,
        callback_channel: Channel,
    ) -> crate::Result<()> {
        Ok(())
    }

    pub fn clear_watch(&self, channel_id: u32) -> crate::Result<()> {
        Ok(())
    }

    pub fn check_permissions(&self) -> crate::Result<PermissionStatus> {
        Ok(PermissionStatus::default())
    }

    pub fn request_permissions(
        &self,
        permissions: Option<Vec<PermissionType>>,
    ) -> crate::Result<PermissionStatus> {
        Ok(PermissionStatus::default())
    }
}

#[derive(Serialize)]
struct WatchPayload {
    options: PositionOptions,
    channel: Channel,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ClearWatchPayload {
    channel_id: u32,
}
