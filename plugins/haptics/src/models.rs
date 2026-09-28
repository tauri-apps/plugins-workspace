// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
/*
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub struct HapticsOptions {
    // TODO: support array to match web api
    pub duration: u32,
}
 */

/// The style of an impact-feedback haptic.
///
/// On iOS this maps directly to a `UIImpactFeedbackGenerator.FeedbackStyle` case. On Android,
/// which has no equivalent system API, each style instead plays a distinct vibration waveform of
/// increasing intensity. Has no effect on desktop platforms. Defaults to `Medium`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum ImpactFeedbackStyle {
    /// A collision between small, light user interface elements.
    Light,
    /// A collision between moderately sized user interface elements.
    #[default]
    Medium,
    /// A collision between large, heavy user interface elements.
    Heavy,
    /// A soft, muted impact between user interface elements.
    Soft,
    /// A sharp, rigid impact between user interface elements.
    Rigid,
}

/// The type of notification feedback, indicating the outcome of a task or action.
///
/// On iOS this maps directly to a `UINotificationFeedbackGenerator.FeedbackType` case. On
/// Android, which has no equivalent system API, each type instead plays a distinct vibration
/// waveform. Has no effect on desktop platforms. Defaults to `Success`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(feature = "specta", derive(specta::Type))]
#[serde(rename_all = "camelCase")]
pub enum NotificationFeedbackType {
    /// A task or action has completed successfully.
    #[default]
    Success,
    /// A task or action has produced a warning.
    Warning,
    /// A task or action has failed.
    Error,
}
