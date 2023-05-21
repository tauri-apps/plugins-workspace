// Copyright 2021 Jonas Kruckenberg
// SPDX-License-Identifier: MIT

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

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
  await window.__TAURI_INVOKE__("plugin:positioner|move_window", {
    position: to,
  });
}
