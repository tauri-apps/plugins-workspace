// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{command, AppHandle, Runtime, State, Window};

use crate::{DeepLink, Result};

#[command]
pub(crate) async fn get_current<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    deep_link: State<'_, DeepLink<R>>,
) -> Result<Option<Vec<url::Url>>> {
    deep_link.get_current()
}
