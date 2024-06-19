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

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.geolocation";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_geolocation);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Geolocation<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "GeolocationPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_geolocation)?;
    Ok(Geolocation(handle))
}

/// Access to the geolocation APIs.
pub struct Geolocation<R: Runtime>(PluginHandle<R>);

// TODO: Position instead of Value
impl<R: Runtime> Geolocation<R> {
    pub fn get_current_position(
        &self,
        options: Option<PositionOptions>,
    ) -> crate::Result<Position> {
        // TODO: We may have to send over None if that's better on Android
        self.0
            .run_mobile_plugin("getCurrentPosition", options.unwrap_or_default())
            .map_err(Into::into)
    }

    // TODO: <F: FnMut(Position) + Send + Sync + 'static>
    pub fn watch_position(
        &self,
        options: PositionOptions,
        callback_channel: Channel,
    ) -> crate::Result<()> {
        self.0
            .run_mobile_plugin(
                "watchPosition",
                WatchPayload {
                    options,
                    channel: callback_channel,
                },
            )
            .map_err(Into::into)
    }

    pub fn clear_watch(&self, channel_id: u32) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("clearWatch", ClearWatchPayload { channel_id })
            .map_err(Into::into)
    }

    pub fn check_permissions(&self) -> crate::Result<PermissionStatus> {
        self.0
            .run_mobile_plugin("checkPermissions", ())
            .map_err(Into::into)
    }

    pub fn request_permissions(
        &self,
        permissions: Option<Vec<PermissionType>>,
    ) -> crate::Result<PermissionStatus> {
        self.0
            .run_mobile_plugin("requestPermissions", permissions)
            .map_err(Into::into)
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
