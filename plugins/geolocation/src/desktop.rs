// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{de::DeserializeOwned, Serialize};
use tauri::{
    ipc::{Channel, InvokeResponseBody},
    plugin::PluginApi,
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

impl<R: Runtime> Geolocation<R> {
    /// Not implemented on desktop platforms; always resolves to a default, zeroed [`Position`] without reading any real location.
    pub fn get_current_position(
        &self,
        _options: Option<PositionOptions>,
    ) -> crate::Result<Position> {
        Ok(Position::default())
    }

    /// Not implemented on desktop platforms. Registers a channel for `channel_id` bookkeeping, but `callback` is never invoked with a real [`WatchEvent`].
    pub fn watch_position<F: Fn(WatchEvent) + Send + Sync + 'static>(
        &self,
        options: PositionOptions,
        callback: F,
    ) -> crate::Result<u32> {
        let channel = Channel::new(move |event| {
            let payload = match event {
                InvokeResponseBody::Json(payload) => serde_json::from_str::<WatchEvent>(&payload)
                    .unwrap_or_else(|error| {
                        WatchEvent::Error(format!(
                            "Couldn't deserialize watch event payload: `{error}`"
                        ))
                    }),
                _ => WatchEvent::Error("Unexpected watch event payload.".to_string()),
            };

            callback(payload);

            Ok(())
        });
        let id = channel.id();

        self.watch_position_inner(options, channel)?;

        Ok(id)
    }

    pub(crate) fn watch_position_inner(
        &self,
        _options: PositionOptions,
        _callback_channel: Channel,
    ) -> crate::Result<()> {
        Ok(())
    }

    /// Not implemented on desktop platforms; always succeeds without doing anything.
    pub fn clear_watch(&self, _channel_id: u32) -> crate::Result<()> {
        Ok(())
    }

    /// Not implemented on desktop platforms; always resolves to the default [`PermissionStatus`] (both permissions in the [`Prompt`](tauri::plugin::PermissionState::Prompt) state).
    pub fn check_permissions(&self) -> crate::Result<PermissionStatus> {
        Ok(PermissionStatus::default())
    }

    /// Not implemented on desktop platforms; always resolves to the default [`PermissionStatus`] without prompting the user.
    pub fn request_permissions(
        &self,
        _permissions: Option<Vec<PermissionType>>,
    ) -> crate::Result<PermissionStatus> {
        Ok(PermissionStatus::default())
    }
}

#[derive(Serialize)]
#[allow(unused)] // TODO:
struct WatchPayload {
    options: PositionOptions,
    channel: Channel,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(unused)] // TODO:
struct ClearWatchPayload {
    channel_id: u32,
}
