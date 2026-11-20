// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

package app.tauri.haptics.patterns

class Pattern (
    val timings: LongArray,
    val amplitudes: IntArray,
    val oldSDKPattern: LongArray
) {}