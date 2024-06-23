// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, AppHandle};

use crate::{HapticsExt, ImpactFeedbackStyle, NotificationFeedbackType, Result};

#[command]
#[specta::specta]
pub(crate) async fn vibrate(app: AppHandle, duration: u32) -> Result<()> {
    app.haptics().vibrate(duration)
}

#[command]
#[specta::specta]
pub(crate) async fn impact_feedback(app: AppHandle, style: ImpactFeedbackStyle) -> Result<()> {
    app.haptics().impact_feedback(style)
}

#[command]
#[specta::specta]
pub(crate) async fn notification_feedback(
    app: AppHandle,
    r#type: NotificationFeedbackType,
) -> Result<()> {
    app.haptics().notification_feedback(r#type)
}

#[command]
#[specta::specta]
pub(crate) async fn selection_feedback(app: AppHandle) -> Result<()> {
    app.haptics().selection_feedback()
}
