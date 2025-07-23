// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{AppHandle, command, Runtime, Window};

use crate::Result;

#[command]
pub(crate) async fn execute<R: Runtime>(
  _app: AppHandle<R>,
  _window: Window<R>,
) -> Result<String> {
  Ok("success".to_string())
}
