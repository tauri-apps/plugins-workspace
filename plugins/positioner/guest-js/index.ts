// Copyright 2021 Jonas Kruckenberg
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Move the window to well-known positions, including positions relative to the tray icon.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'
import type { TrayIconEvent } from '@tauri-apps/api/tray'

/**
 * Well known window positions.
 */
export enum Position {
  /** Top left corner of the screen. */
  TopLeft = 0,
  /** Top right corner of the screen. */
  TopRight,
  /** Bottom left corner of the screen. */
  BottomLeft,
  /** Bottom right corner of the screen. */
  BottomRight,
  /** Top center of the screen. */
  TopCenter,
  /** Bottom center of the screen. */
  BottomCenter,
  /** Vertically centered on the left edge of the screen. */
  LeftCenter,
  /** Vertically centered on the right edge of the screen. */
  RightCenter,
  /** Center of the screen. */
  Center,
  /**
   * Above the tray icon, aligned with its left edge. Requires the `tray-icon` feature and the
   * tray icon's events to be forwarded through {@link handleIconState}.
   */
  TrayLeft,
  /**
   * Directly below the tray icon, aligned with its left edge. Requires the `tray-icon` feature
   * and the tray icon's events to be forwarded through {@link handleIconState}.
   */
  TrayBottomLeft,
  /**
   * Above the tray icon, aligned with its right edge. Requires the `tray-icon` feature and the
   * tray icon's events to be forwarded through {@link handleIconState}.
   */
  TrayRight,
  /**
   * Directly below the tray icon, aligned with its right edge. Requires the `tray-icon` feature
   * and the tray icon's events to be forwarded through {@link handleIconState}.
   */
  TrayBottomRight,
  /**
   * Above the tray icon, horizontally centered on it. Requires the `tray-icon` feature and the
   * tray icon's events to be forwarded through {@link handleIconState}.
   */
  TrayCenter,
  /**
   * Directly below the tray icon, horizontally centered on it. Requires the `tray-icon` feature
   * and the tray icon's events to be forwarded through {@link handleIconState}.
   */
  TrayBottomCenter
}

/**
 * Moves the `Window` to the given {@link Position} using `WindowExt.move_window()`
 * All positions are relative to the **current** screen.
 *
 * @example
 * ```typescript
 * import { moveWindow, Position } from '@tauri-apps/plugin-positioner'
 *
 * await moveWindow(Position.TopRight)
 * ```
 *
 * @param to The {@link Position} to move to.
 * @since 2.0.0
 */
export async function moveWindow(to: Position): Promise<void> {
  await invoke('plugin:positioner|move_window', {
    position: to
  })
}

/**
 * Moves the `Window` to the given {@link Position} using `WindowExt.move_window_constrained()`
 *
 * This move operation constrains the window to the screen dimensions in case of
 * tray-icon positions.
 *
 * @example
 * ```typescript
 * import { moveWindowConstrained, Position } from '@tauri-apps/plugin-positioner'
 *
 * await moveWindowConstrained(Position.TrayCenter)
 * ```
 *
 * @param to The (tray) {@link Position} to move to.
 * @since 2.1.0
 */
export async function moveWindowConstrained(to: Position): Promise<void> {
  await invoke('plugin:positioner|move_window_constrained', {
    position: to
  })
}

/**
 * Reports the tray icon's current position and size so the `Tray*` {@link Position} variants
 * can be resolved.
 *
 * This is an internal helper meant to be called from the tray icon's event handler (the
 * `action` callback passed to `TrayIcon.new()`), forwarding every {@link TrayIconEvent} it
 * receives so that {@link moveWindow} and {@link moveWindowConstrained} can later position the
 * window relative to the tray icon.
 *
 * @example
 * ```typescript
 * import { handleIconState } from '@tauri-apps/plugin-positioner'
 * import { TrayIcon, type TrayIconEvent } from '@tauri-apps/api/tray'
 *
 * const action = async (event: TrayIconEvent) => {
 *   await handleIconState(event)
 * }
 *
 * const tray = await TrayIcon.new({ id: 'main', action })
 * ```
 *
 * @param event The tray icon event to read the position and size from.
 * @since 2.0.0
 */
export async function handleIconState(event: TrayIconEvent): Promise<void> {
  await invoke('plugin:positioner|set_tray_icon_state', {
    position: event.rect.position,
    size: event.rect.size
  })
}
