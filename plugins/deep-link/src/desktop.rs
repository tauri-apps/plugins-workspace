// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<DeepLink<R>> {
    Ok(DeepLink(app.clone()))
}

/// Access to the deep-link APIs.
pub struct DeepLink<R: Runtime>(AppHandle<R>);

impl<R: Runtime> DeepLink<R> {
    pub fn ping(&self, payload: PingRequest) -> crate::Result<PingResponse> {
        Ok(PingResponse {
            value: payload.value,
        })
    }

    pub fn get_last_link(&self) -> crate::Result<Option<String>> {
        Ok(Some("desktop not implemented".to_string()))
    }
}
