// Copyright 2021 Jonas Kruckenberg
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/core";

/**
 * Well known window positions.
 */
export enum Position {
  TopLeft = 0,
  TopRight,
  BottomLeft,
  BottomRight,
  TopCenter,
  BottomCenter,
  LeftCenter,
  RightCenter,
  Center,
  TrayLeft,
  TrayBottomLeft,
  TrayRight,
  TrayBottomRight,
  TrayCenter,
  TrayBottomCenter,
}

/**
 * Moves the `Window` to the given {@link Position} using `WindowExt.move_window()`
 * All positions are relative to the **current** screen.
 *
 * @param to The {@link Position} to move to.
 */
export async function moveWindow(to: Position): Promise<void> {
  await invoke("plugin:positioner|move_window", {
    position: to,
  });
}
