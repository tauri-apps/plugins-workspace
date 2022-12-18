import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';

var LogLevel;
(function (LogLevel) {
    /**
     * The "trace" level.
     *
     * Designates very low priority, often extremely verbose, information.
     */
    LogLevel[LogLevel["Trace"] = 1] = "Trace";
    /**
     * The "debug" level.
     *
     * Designates lower priority information.
     */
    LogLevel[LogLevel["Debug"] = 2] = "Debug";
    /**
     * The "info" level.
     *
     * Designates useful information.
     */
    LogLevel[LogLevel["Info"] = 3] = "Info";
    /**
     * The "warn" level.
     *
     * Designates hazardous situations.
     */
    LogLevel[LogLevel["Warn"] = 4] = "Warn";
    /**
     * The "error" level.
     *
     * Designates very serious errors.
     */
    LogLevel[LogLevel["Error"] = 5] = "Error";
})(LogLevel || (LogLevel = {}));
async function log(level, message, options) {
    var _a, _b;
    const traces = (_a = new Error().stack) === null || _a === void 0 ? void 0 : _a.split("\n").map((line) => line.split("@"));
    const filtered = traces === null || traces === void 0 ? void 0 : traces.filter(([name, location]) => {
        return name.length > 0 && location !== "[native code]";
    });
    const { file, line, ...keyValues } = options !== null && options !== void 0 ? options : {};
    await invoke("plugin:log|log", {
        level,
        message,
        location: (_b = filtered === null || filtered === void 0 ? void 0 : filtered[0]) === null || _b === void 0 ? void 0 : _b.filter((v) => v.length > 0).join("@"),
        file,
        line,
        keyValues,
    });
}
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
async function error(message, options) {
    await log(LogLevel.Error, message, options);
}
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
async function warn(message, options) {
    await log(LogLevel.Warn, message, options);
}
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
async function info(message, options) {
    await log(LogLevel.Info, message, options);
}
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
async function debug(message, options) {
    await log(LogLevel.Debug, message, options);
}
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
async function trace(message, options) {
    await log(LogLevel.Trace, message, options);
}
async function attachConsole() {
    return await listen("log://log", (event) => {
        const payload = event.payload;
        // Strip ANSI escape codes
        const message = payload.message.replace(
        // eslint-disable-next-line no-control-regex
        /[\u001b\u009b][[()#;?]*(?:[0-9]{1,4}(?:;[0-9]{0,4})*)?[0-9A-ORZcf-nqry=><]/g, "");
        switch (payload.level) {
            case LogLevel.Trace:
                console.log(message);
                break;
            case LogLevel.Debug:
                console.debug(message);
                break;
            case LogLevel.Info:
                console.info(message);
                break;
            case LogLevel.Warn:
                console.warn(message);
                break;
            case LogLevel.Error:
                console.error(message);
                break;
            default:
                // eslint-disable-next-line @typescript-eslint/restrict-template-expressions
                throw new Error(`unknown log level ${payload.level}`);
        }
    });
}

export { attachConsole, debug, error, info, trace, warn };
//# sourceMappingURL=index.mjs.map
