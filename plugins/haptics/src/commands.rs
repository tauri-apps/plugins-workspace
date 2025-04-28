// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, AppHandle, Runtime};

use crate::{HapticsExt, ImpactFeedbackStyle, NotificationFeedbackType, Result};

#[command]
pub(crate) async fn vibrate<R: Runtime>(app: AppHandle<R>, duration: u32) -> Result<()> {
    app.haptics().vibrate(duration)
}

#[command]
pub(crate) async fn impact_feedback<R: Runtime>(
    app: AppHandle<R>,
    style: ImpactFeedbackStyle,
) -> Result<()> {
    app.haptics().impact_feedback(style)
}

#[command]
pub(crate) async fn notification_feedback<R: Runtime>(
    app: AppHandle<R>,
    r#type: NotificationFeedbackType,
) -> Result<()> {
    app.haptics().notification_feedback(r#type)
}

#[command]
pub(crate) async fn selection_feedback<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.haptics().selection_feedback()
}
