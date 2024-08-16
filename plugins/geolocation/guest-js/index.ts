// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/* eslint-disable @typescript-eslint/unbound-method */

import { Channel } from "@tauri-apps/api/core";
import { commands, type PositionOptions, type Position } from "./bindings";

export async function watchPosition(
  options: PositionOptions,
  // TODO: This can receive errors too
  cb: (location: Position | string) => void,
): Promise<number> {
  const channel = new Channel<Position>();
  channel.onmessage = cb;
  await commands.watchPosition(options, channel);
  return channel.id;
}

export const {
  getCurrentPosition,
  clearWatch,
  checkPermissions,
  requestPermissions,
} = commands;

export type {
  PermissionState,
  PermissionStatus,
  PermissionType,
  Position,
  PositionOptions,
  Coordinates,
} from "./bindings";

// export { events };
