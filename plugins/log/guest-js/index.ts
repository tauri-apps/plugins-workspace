// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Configure the Tauri application log system, and access the JavaScript-side log functions.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn, type Event } from '@tauri-apps/api/event'

/**
 * Options to associate extra metadata with a log entry.
 */
export interface LogOptions {
  /** The name of the file that emitted the log entry. Included in the log record's target when set. */
  file?: string
  /** The line number in {@linkcode LogOptions.file} that emitted the log entry. */
  line?: number
  /** Additional structured key-value pairs to attach to the log entry. */
  keyValues?: Record<string, string | undefined>
}

/**
 * The verbosity level of a log entry, matching the levels of the Rust `log` crate.
 */
export enum LogLevel {
  /**
   * The "trace" level.
   *
   * Designates very low priority, often extremely verbose, information.
   */
  Trace = 1,
  /**
   * The "debug" level.
   *
   * Designates lower priority information.
   */
  Debug,
  /**
   * The "info" level.
   *
   * Designates useful information.
   */
  Info,
  /**
   * The "warn" level.
   *
   * Designates hazardous situations.
   */
  Warn,
  /**
   * The "error" level.
   *
   * Designates very serious errors.
   */
  Error
}

function getCallerLocation(stack?: string) {
  if (!stack) {
    return
  }

  if (stack.startsWith('Error')) {
    // Assume it's Chromium V8
    //
    // Error
    //     at baz (filename.js:10:15)
    //     at bar (filename.js:6:3)
    //     at foo (filename.js:2:3)
    //     at filename.js:13:1

    const lines = stack.split('\n')
    // Find the third line (caller's caller of the current location)
    const callerLine = lines[3]?.trim()
    if (!callerLine) {
      return
    }

    const regex =
      /at\s+(?<functionName>.*?)\s+\((?<fileName>.*?):(?<lineNumber>\d+):(?<columnNumber>\d+)\)/
    const match = callerLine.match(regex)

    if (match) {
      const { functionName, fileName, lineNumber, columnNumber } =
        match.groups as {
          functionName: string
          fileName: string
          lineNumber: string
          columnNumber: string
        }
      return `${functionName}@${fileName}:${lineNumber}:${columnNumber}`
    } else {
      // Handle cases where the regex does not match (e.g., last line without function name)
      const regexNoFunction =
        /at\s+(?<fileName>.*?):(?<lineNumber>\d+):(?<columnNumber>\d+)/
      const matchNoFunction = callerLine.match(regexNoFunction)
      if (matchNoFunction) {
        const { fileName, lineNumber, columnNumber } =
          matchNoFunction.groups as {
            fileName: string
            lineNumber: string
            columnNumber: string
          }
        return `<anonymous>@${fileName}:${lineNumber}:${columnNumber}`
      }
    }
  } else {
    // Assume it's Webkit JavaScriptCore, example:
    //
    // baz@filename.js:10:24
    // bar@filename.js:6:6
    // foo@filename.js:2:6
    // global code@filename.js:13:4

    const traces = stack.split('\n').map((line) => line.split('@'))
    const filtered = traces.filter(([name, location]) => {
      return name.length > 0 && location !== '[native code]'
    })
    // Find the third line (caller's caller of the current location)
    return filtered[2]?.filter((v) => v.length > 0).join('@')
  }
}

async function log(
  level: LogLevel,
  message: string,
  options?: LogOptions
): Promise<void> {
  const location = getCallerLocation(new Error().stack)

  const { file, line, keyValues } = options ?? {}

  await invoke('plugin:log|log', {
    level,
    message,
    location,
    file,
    line,
    keyValues
  })
}

/**
 * Logs a message at the error level.
 *
 * @example
 * ```typescript
 * import { error } from '@tauri-apps/plugin-log';
 *
 * const err_info = "No connection";
 * const port = 22;
 *
 * error(`Error: ${err_info} on port ${port}`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export async function error(
  message: string,
  options?: LogOptions
): Promise<void> {
  await log(LogLevel.Error, message, options)
}

/**
 * Logs a message at the warn level.
 *
 * @example
 * ```typescript
 * import { warn } from '@tauri-apps/plugin-log';
 *
 * const warn_description = "Invalid Input";
 *
 * warn(`Warning! {warn_description}!`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export async function warn(
  message: string,
  options?: LogOptions
): Promise<void> {
  await log(LogLevel.Warn, message, options)
}

/**
 * Logs a message at the info level.
 *
 * @example
 * ```typescript
 * import { info } from '@tauri-apps/plugin-log';
 *
 * const conn_info = { port: 40, speed: 3.20 };
 *
 * info(`Connected to port {conn_info.port} at {conn_info.speed} Mb/s`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export async function info(
  message: string,
  options?: LogOptions
): Promise<void> {
  await log(LogLevel.Info, message, options)
}

/**
 * Logs a message at the debug level.
 *
 * @example
 * ```typescript
 * import { debug } from '@tauri-apps/plugin-log';
 *
 * const pos = { x: 3.234, y: -1.223 };
 *
 * debug(`New position: x: {pos.x}, y: {pos.y}`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export async function debug(
  message: string,
  options?: LogOptions
): Promise<void> {
  await log(LogLevel.Debug, message, options)
}

/**
 * Logs a message at the trace level.
 *
 * @example
 * ```typescript
 * import { trace } from '@tauri-apps/plugin-log';
 *
 * let pos = { x: 3.234, y: -1.223 };
 *
 * trace(`Position is: x: {pos.x}, y: {pos.y}`);
 * ```
 *
 * @param message the message to log.
 * @param options additional metadata (file, line, key-values) to attach to the log entry.
 * @since 2.0.0
 */
export async function trace(
  message: string,
  options?: LogOptions
): Promise<void> {
  await log(LogLevel.Trace, message, options)
}

interface RecordPayload {
  level: LogLevel
  message: string
}

type LoggerFn = (fn: RecordPayload) => void

/**
 * Attaches a listener for the log, and calls the passed function for each log entry.
 *
 * @example
 * ```typescript
 * import { attachLogger } from '@tauri-apps/plugin-log';
 *
 * const detach = await attachLogger(({ level, message }) => {
 *   console.log(`[${level}] ${message}`);
 * });
 *
 * // detach the listener later
 * detach();
 * ```
 *
 * @param fn a function to call for every log entry emitted by the Rust side.
 * @returns a promise resolving to a function to cancel the listener.
 * @since 2.0.0
 */
export async function attachLogger(fn: LoggerFn): Promise<UnlistenFn> {
  return await listen('log://log', (event: Event<RecordPayload>) => {
    const { level } = event.payload
    let { message } = event.payload

    // Strip ANSI escape codes
    message = message.replace(
      // TODO: Investigate security/detect-unsafe-regex
      // eslint-disable-next-line no-control-regex, security/detect-unsafe-regex
      /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g,
      ''
    )
    fn({ message, level })
  })
}

/**
 * Attaches a listener that writes log entries to the console as they come in.
 *
 * @example
 * ```typescript
 * import { attachConsole } from '@tauri-apps/plugin-log';
 *
 * const detach = await attachConsole();
 *
 * // detach the listener later
 * detach();
 * ```
 *
 * @returns a promise resolving to a function to cancel the listener.
 * @since 2.0.0
 */
export async function attachConsole(): Promise<UnlistenFn> {
  return await attachLogger(({ level, message }: RecordPayload) => {
    switch (level) {
      case LogLevel.Trace:
        console.log(message)
        break
      case LogLevel.Debug:
        console.debug(message)
        break
      case LogLevel.Info:
        console.info(message)
        break
      case LogLevel.Warn:
        console.warn(message)
        break
      case LogLevel.Error:
        console.error(message)
        break
      default:
        // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
        throw new Error(`unknown log level ${level}`)
    }
  })
}
