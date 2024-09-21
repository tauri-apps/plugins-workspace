// Copyright 2021 Jonas Kruckenberg
// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '@tauri-apps/api/core'
import type {
  TrayIconClickEvent,
  TrayIconEnterEvent,
  TrayIconEvent,
  TrayIconLeaveEvent,
  TrayIconMoveEvent
} from '@tauri-apps/api/tray'

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
  TrayBottomCenter
}

/**
 * Moves the `Window` to the given {@link Position} using `WindowExt.move_window()`
 * All positions are relative to the **current** screen.
 *
 * @param to The {@link Position} to move to.
 */
export async function moveWindow(to: Position): Promise<void> {
  await invoke('plugin:positioner|move_window', {
    position: to
  })
}

export async function handleIconState(event: TrayIconEvent): Promise<void> {
  if ('click' in event) {
    await invokeSetTrayIconState(event.click.rect)
  } else if ('enter' in event) {
    await invokeSetTrayIconState(event.enter.rect)
  } else if ('leave' in event) {
    await invokeSetTrayIconState(event.leave.rect)
  } else if ('move' in event) {
    await invokeSetTrayIconState(event.move.rect)
  }
}

async function invokeSetTrayIconState(
  rect:
    | TrayIconClickEvent['rect']
    | TrayIconEnterEvent['rect']
    | TrayIconLeaveEvent['rect']
    | TrayIconMoveEvent['rect']
) {
  await invoke('plugin:positioner|set_tray_icon_state', {
    position: rect.position.Physical,
    size: rect.size.Physical
  })
}
