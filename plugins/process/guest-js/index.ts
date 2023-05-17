// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Perform operations on the current process.
 * @module
 */

import { invoke } from "@tauri-apps/api/tauri";

/**
 * Exits immediately with the given `exitCode`.
 * @example
 * ```typescript
 * import { exit } from '@tauri-plugins/process';
 * await exit(1);
 * ```
 *
 * @param code The exit code to use.
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function exit(code = 0): Promise<void> {
  return invoke("plugin:process|exit", { code });
}

/**
 * Exits the current instance of the app then relaunches it.
 * @example
 * ```typescript
 * import { relaunch } from '@tauri-plugins/process';
 * await relaunch();
 * ```
 *
 * @returns A promise indicating the success or failure of the operation.
 *
 * @since 1.0.0
 */
async function relaunch(): Promise<void> {
  return invoke("plugin:process|restart");
}

export { exit, relaunch };
