// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{de::DeserializeOwned, Serialize};
use tauri::{
    plugin::{PluginApi, PluginHandle},
    AppHandle, Runtime,
};

use crate::models::*;

#[cfg(target_os = "android")]
const PLUGIN_IDENTIFIER: &str = "app.tauri.haptics";

#[cfg(target_os = "ios")]
tauri::ios_plugin_binding!(init_plugin_haptics);

// initializes the Kotlin or Swift plugin classes
pub fn init<R: Runtime, C: DeserializeOwned>(
    _app: &AppHandle<R>,
    api: PluginApi<R, C>,
) -> crate::Result<Haptics<R>> {
    #[cfg(target_os = "android")]
    let handle = api.register_android_plugin(PLUGIN_IDENTIFIER, "HapticsPlugin")?;
    #[cfg(target_os = "ios")]
    let handle = api.register_ios_plugin(init_plugin_haptics)?;
    Ok(Haptics(handle))
}

/// Access to the haptics APIs.
pub struct Haptics<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> Haptics<R> {
    pub fn vibrate(&self, duration: u32) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("vibrate", VibratePayload { duration })
            .map_err(Into::into)
    }

    pub fn impact_feedback(&self, style: ImpactFeedbackStyle) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("impactFeedback", ImpactFeedbackPayload { style })
            .map_err(Into::into)
    }

    pub fn notification_feedback(&self, r#type: NotificationFeedbackType) -> crate::Result<()> {
        self.0
            .run_mobile_plugin(
                "notificationFeedback",
                NotificationFeedbackPayload { r#type },
            )
            .map_err(Into::into)
    }

    pub fn selection_feedback(&self) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("selectionFeedback", ())
            .map_err(Into::into)
    }
}

#[derive(Serialize)]
struct VibratePayload {
    duration: u32,
}

#[derive(Serialize)]
struct ImpactFeedbackPayload {
    style: ImpactFeedbackStyle,
}

#[derive(Serialize)]
struct NotificationFeedbackPayload {
    r#type: NotificationFeedbackType,
}
