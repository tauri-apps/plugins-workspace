import { UnlistenFn } from "@tauri-apps/api/event";
export type LogOptions = {
    file?: string;
    line?: number;
} & Record<string, string | undefined>;
/**
 * Logs a message at the error level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { error } from 'tauri-plugin-log-api';
 *
 * const err_info = "No connection";
 * const port = 22;
 *
 * error(`Error: ${err_info} on port ${port}`);
 * ```
 */
export declare function error(message: string, options?: LogOptions): Promise<void>;
/**
 * Logs a message at the warn level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { warn } from 'tauri-plugin-log-api';
 *
 * const warn_description = "Invalid Input";
 *
 * warn(`Warning! {warn_description}!`);
 * ```
 */
export declare function warn(message: string, options?: LogOptions): Promise<void>;
/**
 * Logs a message at the info level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { info } from 'tauri-plugin-log-api';
 *
 * const conn_info = { port: 40, speed: 3.20 };
 *
 * info(`Connected to port {conn_info.port} at {conn_info.speed} Mb/s`);
 * ```
 */
export declare function info(message: string, options?: LogOptions): Promise<void>;
/**
 * Logs a message at the debug level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { debug } from 'tauri-plugin-log-api';
 *
 * const pos = { x: 3.234, y: -1.223 };
 *
 * debug(`New position: x: {pos.x}, y: {pos.y}`);
 * ```
 */
export declare function debug(message: string, options?: LogOptions): Promise<void>;
/**
 * Logs a message at the trace level.
 *
 * @param message
 *
 * # Examples
 *
 * ```js
 * import { trace } from 'tauri-plugin-log-api';
 *
 * let pos = { x: 3.234, y: -1.223 };
 *
 * trace(`Position is: x: {pos.x}, y: {pos.y}`);
 * ```
 */
export declare function trace(message: string, options?: LogOptions): Promise<void>;
export declare function attachConsole(): Promise<UnlistenFn>;
