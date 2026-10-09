// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Parse arguments from your Command Line Interface.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/**
 * The resolution of a single CLI argument match.
 *
 * @since 2.0.0
 */
interface ArgMatch {
  /**
   * string if takes value
   * boolean if flag
   * string[] or null if takes multiple values
   */
  value: string | boolean | string[] | null
  /**
   * Number of occurrences
   */
  occurrences: number
}

/**
 * The result of matching a subcommand of the CLI definition.
 *
 * @since 2.0.0
 */
interface SubcommandMatch {
  /** The name of the matched subcommand. */
  name: string
  /** The argument matches of the subcommand, resolved the same way as the parent command's matches. */
  matches: CliMatches
}

/**
 * The resolved matches of the CLI arguments and, if any, its matched subcommand.
 *
 * @since 2.0.0
 */
interface CliMatches {
  /** The matched arguments, keyed by argument name. */
  args: Record<string, ArgMatch>
  /** The matched subcommand, or `null` if no subcommand was invoked. */
  subcommand: SubcommandMatch | null
}

/**
 * Parse the arguments provided to the current process and get the matches using the configuration defined [`tauri.cli`](https://tauri.app/v1/api/config/#tauriconfig.cli) in `tauri.conf.json`
 *
 * @example
 * ```typescript
 * import { getMatches } from '@tauri-apps/plugin-cli';
 * const matches = await getMatches();
 * if (matches.subcommand?.name === 'run') {
 *   // `./your-app run $ARGS` was executed
 *   const args = matches.subcommand?.matches.args
 *   if ('debug' in args) {
 *     // `./your-app run --debug` was executed
 *   }
 * } else {
 *   const args = matches.args
 *   // `./your-app $ARGS` was executed
 * }
 * ```
 *
 * @returns A promise resolving to the parsed CLI matches.
 * @since 2.0.0
 */
async function getMatches(): Promise<CliMatches> {
  return await invoke('plugin:cli|cli_matches')
}

export type { ArgMatch, SubcommandMatch, CliMatches }

export { getMatches }
