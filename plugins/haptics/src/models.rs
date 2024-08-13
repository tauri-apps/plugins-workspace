// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use specta::Type;
/*
#[derive(Debug, Clone, Default, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub struct HapticsOptions {
    // TODO: support array to match web api
    pub duration: u32,
}
 */

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum ImpactFeedbackStyle {
    Light,
    #[default]
    Medium,
    Heavy,
    Soft,
    Rigid,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "camelCase")]
pub enum NotificationFeedbackType {
    #[default]
    Success,
    Warning,
    Error,
}
