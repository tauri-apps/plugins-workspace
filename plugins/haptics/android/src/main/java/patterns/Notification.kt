// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.haptics.patterns

val NotificationPatternSuccess = Pattern(
    longArrayOf(0, 40, 100, 40),
    intArrayOf(0, 50, 0, 60),
    longArrayOf(0, 40, 100, 40)
)

val NotificationPatternWarning = Pattern(
    longArrayOf(0, 40, 120, 60),
    intArrayOf(0, 40, 0, 60),
    longArrayOf(0, 40, 120, 60)
)

val NotificationPatternError = Pattern(
    longArrayOf(0, 60, 100, 40, 80, 50),
    intArrayOf(0, 50, 0, 40, 0, 50),
    longArrayOf(0, 60, 100, 40, 80, 50)
)
