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
    /// Triggers a vibration for `duration` milliseconds.
    ///
    /// On iOS this plays a continuous [Core Haptics](https://developer.apple.com/documentation/corehaptics)
    /// pattern when the device supports it, falling back to the system alert vibration
    /// otherwise. On Android it uses [`Vibrator`](https://developer.android.com/reference/android/os/Vibrator).
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
    pub fn vibrate(&self, duration: u32) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("vibrate", VibratePayload { duration })
            .map_err(Into::into)
    }

    /// Triggers an impact-feedback haptic with the given [`ImpactFeedbackStyle`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
    pub fn impact_feedback(&self, style: ImpactFeedbackStyle) -> crate::Result<()> {
        self.0
            .run_mobile_plugin("impactFeedback", ImpactFeedbackPayload { style })
            .map_err(Into::into)
    }

    /// Triggers a notification-feedback haptic for the given [`NotificationFeedbackType`].
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
    pub fn notification_feedback(&self, r#type: NotificationFeedbackType) -> crate::Result<()> {
        self.0
            .run_mobile_plugin(
                "notificationFeedback",
                NotificationFeedbackPayload { r#type },
            )
            .map_err(Into::into)
    }

    /// Triggers a haptic indicating that a selection changed, e.g. when the value of a picker
    /// control changes.
    ///
    /// # Errors
    ///
    /// Returns [`Error::PluginInvoke`](crate::Error::PluginInvoke) if the underlying Android or
    /// iOS plugin invocation fails.
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
