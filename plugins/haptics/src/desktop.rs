// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Runtime};

use crate::models::*;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Haptics<R>> {
    Ok(Haptics(app.clone()))
}

/// Access to the haptics APIs.
pub struct Haptics<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Haptics<R> {
    pub fn vibrate(&self, _duration: u32) -> crate::Result<()> {
        Ok(())
    }

    pub fn impact_feedback(&self, _style: ImpactFeedbackStyle) -> crate::Result<()> {
        Ok(())
    }

    pub fn notification_feedback(&self, _type: NotificationFeedbackType) -> crate::Result<()> {
        Ok(())
    }

    pub fn selection_feedback(&self) -> crate::Result<()> {
        Ok(())
    }
}
