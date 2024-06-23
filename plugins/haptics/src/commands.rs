// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, AppHandle};

use crate::{HapticsExt, Result};

#[command]
#[specta::specta]
pub(crate) async fn vibrate(app: AppHandle, duration: u32) -> Result<()> {
    app.haptics().vibrate(duration)
}
