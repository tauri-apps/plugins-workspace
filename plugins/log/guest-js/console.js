"use strict";
/// <reference lib="esnext" />
var __assign = (this && this.__assign) || function () {
    __assign = Object.assign || function(t) {
        for (var s, i = 1, n = arguments.length; i < n; i++) {
            s = arguments[i];
            for (var p in s) if (Object.prototype.hasOwnProperty.call(s, p))
                t[p] = s[p];
        }
        return t;
    };
    return __assign.apply(this, arguments);
};
var __classPrivateFieldSet = (this && this.__classPrivateFieldSet) || function (receiver, state, value, kind, f) {
    if (kind === "m") throw new TypeError("Private method is not writable");
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a setter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot write private member to an object whose class did not declare it");
    return (kind === "a" ? f.call(receiver, value) : f ? f.value = value : state.set(receiver, value)), value;
};
var __classPrivateFieldGet = (this && this.__classPrivateFieldGet) || function (receiver, state, kind, f) {
    if (kind === "a" && !f) throw new TypeError("Private accessor was defined without a getter");
    if (typeof state === "function" ? receiver !== state || !f : !state.has(receiver)) throw new TypeError("Cannot read private member from an object whose class did not declare it");
    return kind === "m" ? f : kind === "a" ? f.call(receiver) : f ? f.value : state.get(receiver);
};
var __spreadArray = (this && this.__spreadArray) || function (to, from, pack) {
    if (pack || arguments.length === 2) for (var i = 0, l = from.length, ar; i < l; i++) {
        if (ar || !(i in from)) {
            if (!ar) ar = Array.prototype.slice.call(from, 0, i);
            ar[i] = from[i];
        }
    }
    return to.concat(ar || Array.prototype.slice.call(from));
};
var _Console_printFunc, _a;
Object.defineProperty(exports, "__esModule", { value: true });
exports.inspect = exports.Console = exports.getStringWidth = exports.stripVTControlCharacters = void 0;
function isAnyArrayBuffer(value) {
    return value instanceof ArrayBuffer || value instanceof SharedArrayBuffer;
}
function isArgumentsObject(value) {
    return (typeof value == "object" &&
        Object.prototype.toString.call(value) === "[object Arguments]");
}
function isArrayBuffer(value) {
    return value instanceof ArrayBuffer;
}
function isDataView(value) {
    return ArrayBuffer.isView(value) && value instanceof DataView;
}
function isDate(value) {
    return value instanceof Date;
}
function isMap(value) {
    return value instanceof Map;
}
function isPromise(value) {
    return value instanceof Promise;
}
function isRegExp(value) {
    return value instanceof RegExp;
}
function isSet(value) {
    return value instanceof Set;
}
function isGeneratorFunction(value) {
    return (typeof value === "function" &&
        // @ts-expect-error this errors idk
        value[Symbol.toStringTag] === "GeneratorFunction");
}
function isAsyncFunction(value) {
    return (typeof value === "function" &&
        // @ts-expect-error this errors idk
        value[Symbol.toStringTag] === "AsyncFunction");
}
function isTypedArray(value) {
    return (value instanceof Int8Array ||
        value instanceof Uint8Array ||
        value instanceof Uint8ClampedArray ||
        value instanceof Int16Array ||
        value instanceof Uint16Array ||
        value instanceof Int32Array ||
        value instanceof Uint32Array ||
        value instanceof Float32Array ||
        value instanceof Float64Array ||
        value instanceof BigInt64Array ||
        value instanceof BigUint64Array);
}
function isWeakMap(value) {
    return value instanceof WeakMap;
}
function isWeakSet(value) {
    return value instanceof WeakSet;
}
// function isAsyncFunction(value: unknown): value is () => Promise<unknown> {}
// function isBigIntObject(value: unknown): value is BigInt {}
// function isBooleanObject(value: unknown): value is Boolean {}
// function isGeneratorFunction(value: unknown): value is GeneratorFunction {}
// function isNumberObject(value: unknown): value is Number {}
// function isStringObject(value: unknown): value is String {}
// isBoxedPrimitive,
// isMapIterator,
// isModuleNamespaceObject,
// isNativeError,
// isSetIterator,
var kObjectType = 0;
var kArrayType = 1;
var kArrayExtrasType = 2;
var kMinLineLength = 16;
var denoInspectDefaultOptions = {
    indentationLvl: 0,
    currentDepth: 0,
    stylize: function (str) { return str; },
    showHidden: false,
    depth: 4,
    colors: false,
    showProxy: false,
    breakLength: 80,
    escapeSequences: true,
    compact: 3,
    sorted: false,
    getters: false,
    trailingComma: false,
    // TODO(@crowlKats): merge into indentationLvl
    indentLevel: 0,
};
function getDefaultInspectOptions() {
    return __assign({ budget: {}, seen: [], circular: new Map(), quotes: [] }, denoInspectDefaultOptions);
}
var builtInObjectsRegExp = new RegExp("^[A-Z][a-zA-Z0-9]+$");
var builtInObjects = new Set(Object.getOwnPropertyNames(globalThis).filter(function (e) {
    return builtInObjectsRegExp.test(e);
}));
// https://tc39.es/ecma262/#sec-IsHTMLDDA-internal-slot
var isUndetectableObject = function (v) {
    return typeof v === "undefined" && v !== undefined;
};
var strEscapeSequencesReplacer = new RegExp("[\x00-\x1f\x27\x5c\x7f-\x9f]", "g");
var keyStrRegExp = new RegExp("^[a-zA-Z_][a-zA-Z_0-9]*$");
var numberRegExp = new RegExp("^(0|[1-9][0-9]*)$");
// Escaped control characters (plus the single quote and the backslash). Use
// empty strings to fill up unused entries.
// deno-fmt-ignore
var meta = [
    "\\x00",
    "\\x01",
    "\\x02",
    "\\x03",
    "\\x04",
    "\\x05",
    "\\x06",
    "\\x07", // x07
    "\\b",
    "\\t",
    "\\n",
    "\\x0B",
    "\\f",
    "\\r",
    "\\x0E",
    "\\x0F", // x0F
    "\\x10",
    "\\x11",
    "\\x12",
    "\\x13",
    "\\x14",
    "\\x15",
    "\\x16",
    "\\x17", // x17
    "\\x18",
    "\\x19",
    "\\x1A",
    "\\x1B",
    "\\x1C",
    "\\x1D",
    "\\x1E",
    "\\x1F", // x1F
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "\\'",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "", // x2F
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "", // x3F
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "", // x4F
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "\\\\",
    "",
    "",
    "", // x5F
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "", // x6F
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "",
    "\\x7F", // x7F
    "\\x80",
    "\\x81",
    "\\x82",
    "\\x83",
    "\\x84",
    "\\x85",
    "\\x86",
    "\\x87", // x87
    "\\x88",
    "\\x89",
    "\\x8A",
    "\\x8B",
    "\\x8C",
    "\\x8D",
    "\\x8E",
    "\\x8F", // x8F
    "\\x90",
    "\\x91",
    "\\x92",
    "\\x93",
    "\\x94",
    "\\x95",
    "\\x96",
    "\\x97", // x97
    "\\x98",
    "\\x99",
    "\\x9A",
    "\\x9B",
    "\\x9C",
    "\\x9D",
    "\\x9E",
    "\\x9F", // x9F
];
var escapeFn = function (str) { return meta[str.charCodeAt(0)]; };
// Regex used for ansi escape code splitting
// Adopted from https://github.com/chalk/ansi-regex/blob/HEAD/index.js
// License: MIT, authors: @sindresorhus, Qix-, arjunmehta and LitoMore
// Matches all ansi escape code sequences in a string
var ansiPattern = "[\\u001B\\u009B][[\\]()#;?]*" +
    "(?:(?:(?:(?:;[-a-zA-Z\\d\\/#&.:=?%@~_]+)*" +
    "|[a-zA-Z\\d]+(?:;[-a-zA-Z\\d\\/#&.:=?%@~_]*)*)?\\u0007)" +
    "|(?:(?:\\d{1,4}(?:;\\d{0,4})*)?[\\dA-PR-TZcf-ntqry=><~]))";
var ansi = new RegExp(ansiPattern, "g");
/**
 * Remove all VT control characters. Use to estimate displayed string width.
 */
function stripVTControlCharacters(str) {
    return str.replace(ansi, "");
}
exports.stripVTControlCharacters = stripVTControlCharacters;
function getStringWidth(str, removeControlChars) {
    if (removeControlChars === void 0) { removeControlChars = true; }
    var width = 0;
    if (removeControlChars) {
        str = stripVTControlCharacters(str);
    }
    str = str.normalize("NFC");
    for (var _i = 0, str_1 = str; _i < str_1.length; _i++) {
        var char = str_1[_i];
        var code = char.codePointAt(0);
        if (isFullWidthCodePoint(code)) {
            width += 2;
        }
        else if (!isZeroWidthCodePoint(code)) {
            width++;
        }
    }
    return width;
}
exports.getStringWidth = getStringWidth;
var isZeroWidthCodePoint = function (code) {
    return (code <= 0x1f || // C0 control codes
        (code >= 0x7f && code <= 0x9f) || // C1 control codes
        (code >= 0x300 && code <= 0x36f) || // Combining Diacritical Marks
        (code >= 0x200b && code <= 0x200f) || // Modifying Invisible Characters
        // Combining Diacritical Marks for Symbols
        (code >= 0x20d0 && code <= 0x20ff) ||
        (code >= 0xfe00 && code <= 0xfe0f) || // Variation Selectors
        (code >= 0xfe20 && code <= 0xfe2f) || // Combining Half Marks
        (code >= 0xe0100 && code <= 0xe01ef)); // Variation Selectors
};
function isFullWidthCodePoint(code) {
    // Code points are partially derived from:
    // http://www.unicode.org/Public/UNIDATA/EastAsianWidth.txt
    return (code >= 0x1100 &&
        (code <= 0x115f || // Hangul Jamo
            code === 0x2329 || // LEFT-POINTING ANGLE BRACKET
            code === 0x232a || // RIGHT-POINTING ANGLE BRACKET
            // CJK Radicals Supplement .. Enclosed CJK Letters and Months
            (code >= 0x2e80 && code <= 0x3247 && code !== 0x303f) ||
            // Enclosed CJK Letters and Months .. CJK Unified Ideographs Extension A
            (code >= 0x3250 && code <= 0x4dbf) ||
            // CJK Unified Ideographs .. Yi Radicals
            (code >= 0x4e00 && code <= 0xa4c6) ||
            // Hangul Jamo Extended-A
            (code >= 0xa960 && code <= 0xa97c) ||
            // Hangul Syllables
            (code >= 0xac00 && code <= 0xd7a3) ||
            // CJK Compatibility Ideographs
            (code >= 0xf900 && code <= 0xfaff) ||
            // Vertical Forms
            (code >= 0xfe10 && code <= 0xfe19) ||
            // CJK Compatibility Forms .. Small Form Variants
            (code >= 0xfe30 && code <= 0xfe6b) ||
            // Halfwidth and Fullwidth Forms
            (code >= 0xff01 && code <= 0xff60) ||
            (code >= 0xffe0 && code <= 0xffe6) ||
            // Kana Supplement
            (code >= 0x1b000 && code <= 0x1b001) ||
            // Enclosed Ideographic Supplement
            (code >= 0x1f200 && code <= 0x1f251) ||
            // Miscellaneous Symbols and Pictographs 0x1f300 - 0x1f5ff
            // Emoticons 0x1f600 - 0x1f64f
            (code >= 0x1f300 && code <= 0x1f64f) ||
            // CJK Unified Ideographs Extension B .. Tertiary Ideographic Plane
            (code >= 0x20000 && code <= 0x3fffd)));
}
var DEFAULT_INDENT = "  ";
function inspectArgs(args, inspectOptions) {
    if (inspectOptions === void 0) { inspectOptions = {
        __proto__: null,
    }; }
    var ctx = __assign(__assign({}, getDefaultInspectOptions()), inspectOptions);
    var first = args[0];
    var a = 0;
    var string = "";
    if (typeof first == "string" && args.length > 1) {
        a++;
        // Index of the first not-yet-appended character. Use this so we only
        // have to append to `string` when a substitution occurs / at the end.
        var appendedChars = 0;
        for (var i = 0; i < first.length - 1; i++) {
            if (first[i] == "%") {
                var char = first[++i];
                if (a < args.length) {
                    var formattedArg = null;
                    if (char == "s") {
                        // Format as a string.
                        formattedArg = String(args[a++]);
                    }
                    else if (["d", "i"].includes(char)) {
                        // Format as an integer.
                        var value = args[a++];
                        if (typeof value == "bigint") {
                            formattedArg = "".concat(value, "n");
                        }
                        else if (typeof value == "number") {
                            formattedArg = "".concat(Number.parseInt(String(value)));
                        }
                        else {
                            formattedArg = "NaN";
                        }
                    }
                    else if (char == "f") {
                        // Format as a floating point value.
                        var value = args[a++];
                        if (typeof value == "number") {
                            formattedArg = "".concat(value);
                        }
                        else {
                            formattedArg = "NaN";
                        }
                    }
                    else if (["O", "o"].includes(char)) {
                        // Format as an object.
                        formattedArg = formatValue(ctx, args[a++], 0);
                    }
                    else if (char == "c") {
                        formattedArg = "";
                    }
                    if (formattedArg != null) {
                        string += first.slice(appendedChars, i - 1) + formattedArg;
                        appendedChars = i + 1;
                    }
                }
                if (char == "%") {
                    string += first.slice(appendedChars, i - 1) + "%";
                    appendedChars = i + 1;
                }
            }
        }
        string += first.slice(appendedChars);
    }
    for (; a < args.length; a++) {
        if (a > 0) {
            string += " ";
        }
        if (typeof args[a] == "string") {
            string += args[a];
        }
        else {
            // Use default maximum depth for null or undefined arguments.
            string += formatValue(ctx, args[a], 0);
        }
    }
    if (ctx.indentLevel > 0) {
        var groupIndent = DEFAULT_INDENT.repeat(ctx.indentLevel);
        string = groupIndent + string.replaceAll("\n", "\n".concat(groupIndent));
    }
    return string;
}
function formatValue(ctx, value, recurseTimes, typedArray) {
    // Primitive types cannot have properties.
    if (typeof value !== "object" &&
        typeof value !== "function" &&
        !isUndetectableObject(value)) {
        return formatPrimitive(ctx.stylize, value, ctx);
    }
    if (value === null) {
        return ctx.stylize("null", "null");
    }
    // Using an array here is actually better for the average case than using
    // a Set. `seen` will only check for the depth and will never grow too large.
    if (ctx.seen.includes(value)) {
        var index = 1;
        if (ctx.circular === undefined) {
            ctx.circular = new Map();
            ctx.circular.set(value, index);
        }
        else {
            index = ctx.circular.get(value);
            if (index === undefined) {
                index = ctx.circular.size + 1;
                ctx.circular.set(value, index);
            }
        }
        return ctx.stylize("[Circular *".concat(index, "]"), "special");
    }
    return formatRaw(ctx, value, recurseTimes, typedArray);
}
var formatPrimitiveRegExp = new RegExp("(?<=\n)");
function formatPrimitive(fn, value, ctx) {
    if (typeof value === "string") {
        if (
        // TODO(BridgeAR): Add unicode support. Use the readline getStringWidth
        // function.
        value.length > kMinLineLength &&
            value.length > ctx.breakLength - ctx.indentationLvl - 4) {
            return value
                .split(formatPrimitiveRegExp)
                .map(function (line) { return fn(quoteString(line, ctx), "string"); })
                .join(" +\n".concat(" ".repeat(ctx.indentationLvl + 2)));
        }
        return fn(quoteString(value, ctx), "string");
    }
    if (typeof value === "number") {
        return formatNumber(fn, value);
    }
    if (typeof value === "bigint") {
        return formatBigInt(fn, value);
    }
    if (typeof value === "boolean") {
        return fn("".concat(value), "boolean");
    }
    if (typeof value === "undefined") {
        return fn("undefined", "undefined");
    }
    // es6 symbol primitive
    return fn(maybeQuoteSymbol(value, ctx), "symbol");
}
function formatNumber(fn, value) {
    // Format -0 as '-0'. Checking `value === -0` won't distinguish 0 from -0.
    return fn(Object.is(value, -0) ? "-0" : "".concat(value), "number");
}
function formatBigInt(fn, value) {
    return fn("".concat(value, "n"), "bigint");
}
var QUOTE_SYMBOL_REG = new RegExp(/^[a-zA-Z_][a-zA-Z_.0-9]*$/);
function maybeQuoteSymbol(symbol, ctx) {
    var description = symbol.description;
    if (description === undefined) {
        return symbol.toString();
    }
    if (QUOTE_SYMBOL_REG.test(description)) {
        return symbol.toString();
    }
    return "Symbol(".concat(quoteString(description, ctx), ")");
}
/** Surround the string in quotes.
 *
 * The quote symbol is chosen by taking the first of the `QUOTES` array which
 * does not occur in the string. If they all occur, settle with `QUOTES[0]`.
 *
 * Insert a backslash before any occurrence of the chosen quote symbol and
 * before any backslash.
 */
function quoteString(string, ctx) {
    var _b;
    var quote = (_b = ctx.quotes.find(function (c) { return !string.includes(c); })) !== null && _b !== void 0 ? _b : ctx.quotes[0];
    var escapePattern = new RegExp("(?=[".concat(quote, "\\\\])"), "g");
    string = string.replace(escapePattern, "\\");
    if (ctx.escapeSequences) {
        string = replaceEscapeSequences(string);
    }
    return "".concat(quote).concat(string).concat(quote);
}
var ESCAPE_PATTERN = new RegExp(/([\b\f\n\r\t\v])/g);
var ESCAPE_MAP = Object.freeze({
    "\b": "\\b",
    "\f": "\\f",
    "\n": "\\n",
    "\r": "\\r",
    "\t": "\\t",
    "\v": "\\v",
});
var ESCAPE_PATTERN2 = new RegExp("[\x00-\x1f\x7f-\x9f]", "g");
// Replace escape sequences that can modify output.
function replaceEscapeSequences(string) {
    return string
        .replace(ESCAPE_PATTERN, function (c) { return ESCAPE_MAP[c]; })
        .replace(ESCAPE_PATTERN2, function (c) { return "\\x" + c.charCodeAt(0).toString(16).padStart(2, "0"); });
}
function formatSet(value, ctx, _ignored, recurseTimes) {
    ctx.indentationLvl += 2;
    var values = __spreadArray([], value, true);
    var valLen = value.size;
    var len = Math.min(100, valLen);
    var remaining = valLen - len;
    var output = [];
    for (var i = 0; i < len; i++) {
        output.push(formatValue(ctx, values[i], recurseTimes));
    }
    if (remaining > 0) {
        output.push("... ".concat(remaining, " more item").concat(remaining > 1 ? "s" : ""));
    }
    ctx.indentationLvl -= 2;
    return output;
}
function formatMap(value, ctx, _ignored, recurseTimes) {
    ctx.indentationLvl += 2;
    var values = __spreadArray([], value, true);
    var valLen = value.size;
    var len = Math.min(100, valLen);
    var remaining = valLen - len;
    var output = [];
    for (var i = 0; i < len; i++) {
        output.push("".concat(formatValue(ctx, values[i][0], recurseTimes), " => ").concat(formatValue(ctx, values[i][1], recurseTimes)));
    }
    if (remaining > 0) {
        output.push("... ".concat(remaining, " more item").concat(remaining > 1 ? "s" : ""));
    }
    ctx.indentationLvl -= 2;
    return output;
}
function formatArray(ctx, value, recurseTimes) {
    var valLen = value.length;
    var len = Math.min(100, valLen);
    var remaining = valLen - len;
    var output = [];
    for (var i = 0; i < len; i++) {
        // Special handle sparse arrays.
        if (!Object.hasOwn(value, i)) {
            return formatSpecialArray(ctx, value, recurseTimes, len, output, i);
        }
        // @ts-expect-error this is fine
        output.push(formatProperty(ctx, value, recurseTimes, i, kArrayType));
    }
    if (remaining > 0) {
        output.push("... ".concat(remaining, " more item").concat(remaining > 1 ? "s" : ""));
    }
    return output;
}
// The array is sparse and/or has extra keys
function formatSpecialArray(ctx, value, recurseTimes, maxLength, output, i) {
    var keys = Object.keys(value);
    var index = i;
    for (; i < keys.length && output.length < maxLength; i++) {
        var key = keys[i];
        var tmp = +key;
        // Arrays can only have up to 2^32 - 1 entries
        if (tmp > Math.pow(2, 32) - 2) {
            break;
        }
        if ("".concat(index) !== key) {
            if (!numberRegExp.test(key)) {
                break;
            }
            var emptyItems = tmp - index;
            var ending = emptyItems > 1 ? "s" : "";
            var message = "<".concat(emptyItems, " empty item").concat(ending, ">");
            output.push(ctx.stylize(message, "undefined"));
            index = tmp;
            if (output.length === maxLength) {
                break;
            }
        }
        // @ts-expect-error this is fine
        output.push(formatProperty(ctx, value, recurseTimes, key, kArrayType));
        index++;
    }
    var remaining = value.length - index;
    if (output.length !== maxLength) {
        if (remaining > 0) {
            var ending = remaining > 1 ? "s" : "";
            var message = "<".concat(remaining, " empty item").concat(ending, ">");
            output.push(ctx.stylize(message, "undefined"));
        }
    }
    else if (remaining > 0) {
        output.push("... ".concat(remaining, " more item").concat(remaining > 1 ? "s" : ""));
    }
    return output;
}
function formatTypedArray(value, length, ctx, _ignored, recurseTimes) {
    var maxLength = Math.min(100, length);
    var remaining = value.length - maxLength;
    var output = [];
    // @ts-expect-error this errors idk
    var elementFormatter = value.length > 0 && typeof value[0] === "number"
        ? formatNumber
        : formatBigInt;
    for (var i = 0; i < maxLength; ++i) {
        output[i] = elementFormatter(ctx.stylize, value[i]);
    }
    if (remaining > 0) {
        output[maxLength] = "... ".concat(remaining, " more item").concat(remaining > 1 ? "s" : "");
    }
    if (ctx.showHidden) {
        // .buffer goes last, it's not a primitive like the others.
        // All besides `BYTES_PER_ELEMENT` are actually getters.
        ctx.indentationLvl += 2;
        for (var _i = 0, _b = [
            "BYTES_PER_ELEMENT",
            "length",
            "byteLength",
            "byteOffset",
            "buffer",
        ]; _i < _b.length; _i++) {
            var key = _b[_i];
            var str = formatValue(ctx, value[key], recurseTimes, true);
            output.push("[".concat(key, "]: ").concat(str));
        }
        ctx.indentationLvl -= 2;
    }
    return output;
}
var arrayBufferRegExp = new RegExp("(.{2})", "g");
function formatArrayBuffer(ctx, value, _ignored) {
    var valLen;
    try {
        valLen = value.byteLength;
    }
    catch (_b) {
        valLen = getSharedArrayBufferByteLength(value);
    }
    var len = Math.min(100, valLen);
    var buffer;
    try {
        buffer = new Uint8Array(value, 0, len);
    }
    catch (_c) {
        return [ctx.stylize("(detached)", "special")];
    }
    var str = hexSlice(buffer).replace(arrayBufferRegExp, "$1 ").trim();
    var remaining = valLen - len;
    if (remaining > 0) {
        str += " ... ".concat(remaining, " more byte").concat(remaining > 1 ? "s" : "");
    }
    return ["".concat(ctx.stylize("[Uint8Contents]", "special"), ": <").concat(str, ">")];
}
function formatPromise(ctx, value, recurseTimes) {
    return ["Promise"];
}
function formatWeakCollection(ctx) {
    return [ctx.stylize("<items unknown>", "special")];
}
function formatWeakSet(ctx, value, recurseTimes) {
    return ["WeakSet"];
}
function formatWeakMap(ctx, value, recurseTimes) {
    return ["WeakMap"];
}
var hexSliceLookupTable = (function () {
    var alphabet = "0123456789abcdef";
    var table = [];
    for (var i = 0; i < 16; ++i) {
        var i16 = i * 16;
        for (var j = 0; j < 16; ++j) {
            table[i16 + j] = alphabet[i] + alphabet[j];
        }
    }
    return table;
})();
function hexSlice(buf, start, end) {
    var len = buf.length;
    if (!start || start < 0) {
        start = 0;
    }
    if (!end || end < 0 || end > len) {
        end = len;
    }
    var out = "";
    for (var i = start; i < end; ++i) {
        out += hexSliceLookupTable[buf[i]];
    }
    return out;
}
// https://tc39.es/ecma262/#sec-get-sharedarraybuffer.prototype.bytelength
var _getSharedArrayBufferByteLength;
function getSharedArrayBufferByteLength(value) {
    // TODO(kt3k): add SharedArrayBuffer to primordials
    // @ts-expect-error this is fine
    _getSharedArrayBufferByteLength !== null && _getSharedArrayBufferByteLength !== void 0 ? _getSharedArrayBufferByteLength : (_getSharedArrayBufferByteLength = Object.getOwnPropertyDescriptor(SharedArrayBuffer.prototype, "byteLength").get);
    return _getSharedArrayBufferByteLength.call(value);
}
// Look up the keys of the object.
function getKeys(value, showHidden) {
    var keys;
    var symbols = Object.getOwnPropertySymbols(value);
    if (showHidden) {
        keys = Object.getOwnPropertyNames(value);
        if (symbols.length !== 0) {
            keys.push.apply(keys, symbols);
        }
    }
    else {
        // This might throw if `value` is a Module Namespace Object from an
        // unevaluated module, but we don't want to perform the actual type
        // check because it's expensive.
        // TODO(devsnek): track https://github.com/tc39/ecma262/issues/1209
        // and modify this logic as needed.
        try {
            keys = Object.keys(value);
        }
        catch (err) {
            // assert(
            //   isNativeError(err) &&
            //     err.name === "ReferenceError" &&
            //     isModuleNamespaceObject(value),
            // );
            keys = Object.getOwnPropertyNames(value);
        }
        if (symbols.length !== 0) {
            var filter = function (key) {
                return Object.prototype.propertyIsEnumerable.call(value, key);
            };
            keys.push.apply(keys, symbols.filter(filter));
        }
    }
    return keys;
}
function getPrefix(constructor, tag, fallback, size) {
    if (size === void 0) { size = ""; }
    if (constructor === null) {
        if (tag !== "" && fallback !== tag) {
            return "[".concat(fallback).concat(size, ": null prototype] [").concat(tag, "] ");
        }
        return "[".concat(fallback).concat(size, ": null prototype] ");
    }
    if (tag !== "" && constructor !== tag) {
        return "".concat(constructor).concat(size, " [").concat(tag, "] ");
    }
    return "".concat(constructor).concat(size, " ");
}
function getCtxStyle(value, constructor, tag) {
    var fallback = "";
    if (constructor === null) {
        if (fallback === tag) {
            fallback = "Object";
        }
    }
    return getPrefix(constructor, tag, fallback);
}
function formatRaw(ctx, value, recurseTimes, typedArray) {
    var keys = [];
    var protoProps;
    if (ctx.showHidden && (recurseTimes <= ctx.depth || ctx.depth === null)) {
        protoProps = [];
    }
    var constructor = getConstructorName(value, ctx, recurseTimes, protoProps);
    // Reset the variable to check for this later on.
    if (protoProps !== undefined && protoProps.length === 0) {
        protoProps = undefined;
    }
    // @ts-expect-error this is fine
    var _tag = value[Symbol.toStringTag];
    // Only list the tag in case it's non-enumerable / not an own property.
    // Otherwise we'd print this twice.
    if (typeof _tag !== "string") {
        _tag = "";
    }
    var tag = _tag;
    var base = "";
    var formatter = function () { return []; };
    var braces = [];
    var noIterator = true;
    var i = 0;
    var extrasType = kObjectType;
    // Iterators and the rest are split to reduce checks.
    // We have to check all values in case the constructor is set to null.
    // Otherwise it would not possible to identify all types properly.
    if (Reflect.has(value, Symbol.iterator) || constructor === null) {
        noIterator = false;
        if (Array.isArray(value)) {
            // Only set the constructor for non ordinary ("Array [...]") arrays.
            var prefix = constructor !== "Array" || tag !== ""
                ? getPrefix(constructor, tag, "Array", "(".concat(value.length, ")"))
                : "";
            keys = Object.getOwnPropertyNames(value);
            braces = ["".concat(prefix, "["), "]"];
            if (value.length === 0 && keys.length === 0 && protoProps === undefined) {
                return "".concat(braces[0], "]");
            }
            extrasType = kArrayExtrasType;
            formatter = formatArray;
        }
        else if (isSet(value)) {
            var size = value.size;
            var prefix = getPrefix(constructor, tag, "Set", "(".concat(size, ")"));
            keys = getKeys(value, ctx.showHidden);
            formatter =
                constructor !== null
                    ? formatSet.bind(null, value)
                    : formatSet.bind(null, new Set(value.values()));
            if (size === 0 && keys.length === 0 && protoProps === undefined) {
                return "".concat(prefix, "{}");
            }
            braces = ["".concat(prefix, "{"), "}"];
        }
        else if (isMap(value)) {
            var size = value.size;
            var prefix = getPrefix(constructor, tag, "Map", "(".concat(size, ")"));
            keys = getKeys(value, ctx.showHidden);
            formatter =
                constructor !== null
                    ? formatMap.bind(null, value)
                    : formatMap.bind(null, new Map(value.entries()));
            if (size === 0 && keys.length === 0 && protoProps === undefined) {
                return "".concat(prefix, "{}");
            }
            braces = ["".concat(prefix, "{"), "}"];
        }
        else if (isTypedArray(value)) {
            keys = Object.getOwnPropertyNames(value);
            var fallback = "";
            var size = value.length;
            var prefix = getPrefix(constructor, tag, fallback, "(".concat(size, ")"));
            braces = ["".concat(prefix, "["), "]"];
            if (value.length === 0 && keys.length === 0 && !ctx.showHidden) {
                return "".concat(braces[0], "]");
            }
            // Special handle the value. The original value is required below. The
            // bound function is required to reconstruct missing information.
            formatter = formatTypedArray.bind(null, value, size);
            extrasType = kArrayExtrasType;
        } /*else if (isMapIterator(value)) {
          keys = getKeys(value, ctx.showHidden);
          braces = getIteratorBraces("Map", tag);
          // Add braces to the formatter parameters.
          formatter = formatIterator.bind(null, braces);
        } else if (isSetIterator(value)) {
          keys = getKeys(value, ctx.showHidden);
          braces = getIteratorBraces("Set", tag);
          // Add braces to the formatter parameters.
          formatter = FunctionPrototypeBind(formatIterator, null, braces);
        }*/
        else {
            noIterator = true;
        }
    }
    if (noIterator) {
        keys = getKeys(value, ctx.showHidden);
        braces = ["{", "}"];
        if (constructor === "Object") {
            if (isArgumentsObject(value)) {
                braces[0] = "[Arguments] {";
            }
            else if (tag !== "") {
                braces[0] = "".concat(getPrefix(constructor, tag, "Object"), "{");
            }
            if (keys.length === 0 && protoProps === undefined) {
                return "".concat(braces[0], "}");
            }
        }
        else if (typeof value === "function") {
            base = getFunctionBase(value, constructor, tag);
            if (keys.length === 0 && protoProps === undefined) {
                return ctx.stylize(base, "special");
            }
        }
        else if (isRegExp(value)) {
            // Make RegExps say that they are RegExps
            base = (constructor !== null ? value : new RegExp(value)).toString();
            var prefix = getPrefix(constructor, tag, "RegExp");
            if (prefix !== "RegExp ") {
                base = "".concat(prefix).concat(base);
            }
            if ((keys.length === 0 && protoProps === undefined) ||
                (recurseTimes > ctx.depth && ctx.depth !== null)) {
                return ctx.stylize(base, "regexp");
            }
        }
        else if (isDate(value)) {
            if (Number.isNaN(value.getTime())) {
                return ctx.stylize("Invalid Date", "date");
            }
            else {
                base = value.toISOString();
                if (keys.length === 0 && protoProps === undefined) {
                    return ctx.stylize(base, "date");
                }
            }
        }
        else if (
        // @ts-expect-error this is fine
        typeof globalThis.Temporal !== "undefined" &&
            (Object.prototype.isPrototypeOf.call(
            // @ts-expect-error this is fine
            globalThis.Temporal.Instant.prototype, value) ||
                Object.prototype.isPrototypeOf.call(
                // @ts-expect-error this is fine
                globalThis.Temporal.ZonedDateTime.prototype, value) ||
                Object.prototype.isPrototypeOf.call(
                // @ts-expect-error this is fine
                globalThis.Temporal.PlainDate.prototype, value) ||
                Object.prototype.isPrototypeOf.call(
                // @ts-expect-error this is fine
                globalThis.Temporal.PlainTime.prototype, value) ||
                Object.prototype.isPrototypeOf.call(
                // @ts-expect-error this is fine
                globalThis.Temporal.PlainDateTime.prototype, value) ||
                Object.prototype.isPrototypeOf.call(
                // @ts-expect-error this is fine
                globalThis.Temporal.PlainYearMonth.prototype, value) ||
                Object.prototype.isPrototypeOf.call(
                // @ts-expect-error this is fine
                globalThis.Temporal.PlainMonthDay.prototype, value) ||
                Object.prototype.isPrototypeOf.call(
                // @ts-expect-error this is fine
                globalThis.Temporal.Duration.prototype, value) ||
                Object.prototype.isPrototypeOf.call(
                // @ts-expect-error this is fine
                globalThis.Temporal.TimeZone.prototype, value) ||
                Object.prototype.isPrototypeOf.call(
                // @ts-expect-error this is fine
                globalThis.Temporal.Calendar.prototype, value))) {
            // Temporal is not available in primordials yet
            // deno-lint-ignore prefer-primordials
            return ctx.stylize(value.toString(), "temporal");
        } /*else if (
          isNativeError(value) ||
          Object.prototype.isPrototypeOf.call(ErrorPrototype, value)
        ) {
          const error = value;
          base = inspectError(error, ctx);
          if (keys.length === 0 && protoProps === undefined) {
            return base;
          }
        } */
        else if (isAnyArrayBuffer(value)) {
            // Fast path for ArrayBuffer and SharedArrayBuffer.
            // Can't do the same for DataView because it has a non-primitive
            // .buffer property that we need to recurse for.
            var arrayType = isArrayBuffer(value)
                ? "ArrayBuffer"
                : "SharedArrayBuffer";
            var prefix = getPrefix(constructor, tag, arrayType);
            if (typedArray === undefined) {
                formatter = formatArrayBuffer;
            }
            else if (keys.length === 0 && protoProps === undefined) {
                return (prefix +
                    "{ byteLength: ".concat(formatNumber(ctx.stylize, value.byteLength), " }"));
            }
            braces[0] = "".concat(prefix, "{");
            keys.unshift("byteLength");
        }
        else if (isDataView(value)) {
            braces[0] = "".concat(getPrefix(constructor, tag, "DataView"), "{");
            // .buffer goes last, it's not a primitive like the others.
            keys.unshift("byteLength", "byteOffset", "buffer");
        }
        else if (isPromise(value)) {
            braces[0] = "".concat(getPrefix(constructor, tag, "Promise"), "{");
            formatter = formatPromise;
        }
        else if (isWeakSet(value)) {
            braces[0] = "".concat(getPrefix(constructor, tag, "WeakSet"), "{");
            formatter = ctx.showHidden ? formatWeakSet : formatWeakCollection;
        }
        else if (isWeakMap(value)) {
            braces[0] = "".concat(getPrefix(constructor, tag, "WeakMap"), "{");
            formatter = ctx.showHidden ? formatWeakMap : formatWeakCollection;
        } /* else if (isModuleNamespaceObject(value)) {
          braces[0] = `${getPrefix(constructor, tag, "Module")}{`;
          // Special handle keys for namespace objects.
          formatter = FunctionPrototypeBind(formatNamespaceObject, null, keys);
        } */
        else {
            if (keys.length === 0 && protoProps === undefined) {
                // TODO(wafuwafu13): Implement
                // if (isExternal(value)) {
                //   const address = getExternalValue(value).toString(16);
                //   return ctx.stylize(`[External: ${address}]`, 'special');
                // }
                return "".concat(getCtxStyle(value, constructor, tag), "{}");
            }
            braces[0] = "".concat(getCtxStyle(value, constructor, tag), "{");
        }
    }
    if (recurseTimes > ctx.depth && ctx.depth !== null) {
        var constructorName = getCtxStyle(value, constructor, tag).slice(0, -1);
        if (constructor !== null) {
            constructorName = "[".concat(constructorName, "]");
        }
        return ctx.stylize(constructorName, "special");
    }
    recurseTimes += 1;
    ctx.seen.push(value);
    ctx.currentDepth = recurseTimes;
    var output;
    try {
        output = formatter(ctx, value, recurseTimes);
        for (i = 0; i < keys.length; i++) {
            output.push(
            // @ts-expect-error this is fine
            formatProperty(ctx, value, recurseTimes, keys[i], extrasType));
        }
        if (protoProps !== undefined) {
            output.push.apply(output, protoProps);
        }
    }
    catch (error) {
        // TODO(wafuwafu13): Implement stack overflow check
        return ctx.stylize("[Internal Formatting Error] ".concat(error.stack), "internalError");
    }
    if (ctx.circular !== undefined) {
        var index = ctx.circular.get(value);
        if (index !== undefined) {
            var reference = ctx.stylize("<ref *".concat(index, ">"), "special");
            // Add reference always to the very beginning of the output.
            if (ctx.compact !== true) {
                base = base === "" ? reference : "".concat(reference, " ").concat(base);
            }
            else {
                braces[0] = "".concat(reference, " ").concat(braces[0]);
            }
        }
    }
    ctx.seen.pop();
    if (ctx.sorted) {
        var comparator = ctx.sorted === true ? undefined : ctx.sorted;
        if (extrasType === kObjectType) {
            output = output.sort(comparator);
        }
        else if (keys.length > 1) {
            var sorted = output.slice(output.length - keys.length).sort(comparator);
            output.splice.apply(output, __spreadArray([output.length - keys.length, keys.length], sorted, false));
        }
    }
    var res = reduceToSingleString(ctx, output, base, braces, extrasType, recurseTimes, [value]);
    var budget = ctx.budget[ctx.indentationLvl] || 0;
    var newLength = budget + res.length;
    ctx.budget[ctx.indentationLvl] = newLength;
    // If any indentationLvl exceeds this limit, limit further inspecting to the
    // minimum. Otherwise the recursive algorithm might continue inspecting the
    // object even though the maximum string size (~2 ** 28 on 32 bit systems and
    // ~2 ** 30 on 64 bit systems) exceeded. The actual output is not limited at
    // exactly 2 ** 27 but a bit higher. This depends on the object shape.
    // This limit also makes sure that huge objects don't block the event loop
    // significantly.
    if (newLength > Math.pow(2, 27)) {
        ctx.depth = -1;
    }
    return res;
}
function reduceToSingleString(ctx, output, base, braces, extrasType, recurseTimes, value) {
    if (ctx.compact !== true) {
        if (typeof ctx.compact === "number" && ctx.compact >= 1) {
            // Memorize the original output length. In case the output is grouped,
            // prevent lining up the entries on a single line.
            var entries = output.length;
            // Group array elements together if the array contains at least six
            // separate entries.
            if (extrasType === kArrayExtrasType && entries > 6) {
                output = groupArrayElements(ctx, output, value);
            }
            // `ctx.currentDepth` is set to the most inner depth of the currently
            // inspected object part while `recurseTimes` is the actual current depth
            // that is inspected.
            //
            // Example:
            //
            // const a = { first: [ 1, 2, 3 ], second: { inner: [ 1, 2, 3 ] } }
            //
            // The deepest depth of `a` is 2 (a.second.inner) and `a.first` has a max
            // depth of 1.
            //
            // Consolidate all entries of the local most inner depth up to
            // `ctx.compact`, as long as the properties are smaller than
            // `ctx.breakLength`.
            if (ctx.currentDepth - recurseTimes < ctx.compact &&
                entries === output.length) {
                // Line up all entries on a single line in case the entries do not
                // exceed `breakLength`. Add 10 as constant to start next to all other
                // factors that may reduce `breakLength`.
                var start = output.length +
                    ctx.indentationLvl +
                    braces[0].length +
                    base.length +
                    10;
                if (isBelowBreakLength(ctx, output, start, base)) {
                    var joinedOutput = output.join(", ");
                    if (!joinedOutput.includes("\n")) {
                        return ("".concat(base ? "".concat(base, " ") : "").concat(braces[0], " ").concat(joinedOutput) +
                            " ".concat(braces[1]));
                    }
                }
            }
        }
        // Line up each entry on an individual line.
        var indentation_1 = "\n".concat(" ".repeat(ctx.indentationLvl));
        return ("".concat(base ? "".concat(base, " ") : "").concat(braces[0]).concat(indentation_1, "  ") +
            "".concat(output.join(",".concat(indentation_1, "  "))).concat(ctx.trailingComma ? "," : "").concat(indentation_1).concat(braces[1]));
    }
    // Line up all entries on a single line in case the entries do not exceed
    // `breakLength`.
    if (isBelowBreakLength(ctx, output, 0, base)) {
        return ("".concat(braces[0]).concat(base ? " ".concat(base) : "", " ").concat(output.join(", "), " ") + braces[1]);
    }
    var indentation = " ".repeat(ctx.indentationLvl);
    // If the opening "brace" is too large, like in the case of "Set {",
    // we need to force the first item to be on the next line or the
    // items will not line up correctly.
    var ln = base === "" && braces[0].length === 1
        ? " "
        : "".concat(base ? " ".concat(base) : "", "\n").concat(indentation, "  ");
    // Line up each entry on an individual line.
    return "".concat(braces[0]).concat(ln).concat(output.join(",\n".concat(indentation, "  ")), " ").concat(braces[1]);
}
function isBelowBreakLength(ctx, output, start, base) {
    // Each entry is separated by at least a comma. Thus, we start with a total
    // length of at least `output.length`. In addition, some cases have a
    // whitespace in-between each other that is added to the total as well.
    // TODO(BridgeAR): Add unicode support. Use the readline getStringWidth
    // function. Check the performance overhead and make it an opt-in in case it's
    // significant.
    var totalLength = output.length + start;
    if (totalLength + output.length > ctx.breakLength) {
        return false;
    }
    for (var i = 0; i < output.length; i++) {
        totalLength += output[i].length;
        if (totalLength > ctx.breakLength) {
            return false;
        }
    }
    // Do not line up properties on the same line if `base` contains line breaks.
    return base === "" || !base.includes("\n");
}
function groupArrayElements(ctx, output, value) {
    var totalLength = 0;
    var maxLength = 0;
    var i = 0;
    var outputLength = output.length;
    if (100 < output.length) {
        // This makes sure the "... n more items" part is not taken into account.
        outputLength--;
    }
    var separatorSpace = 2; // Add 1 for the space and 1 for the separator.
    var dataLen = [];
    // Calculate the total length of all output entries and the individual max
    // entries length of all output entries. We have to remove colors first,
    // otherwise the length would not be calculated properly.
    for (; i < outputLength; i++) {
        var len = getStringWidth(output[i], ctx.colors);
        dataLen[i] = len;
        totalLength += len + separatorSpace;
        if (maxLength < len) {
            maxLength = len;
        }
    }
    // Add two to `maxLength` as we add a single whitespace character plus a comma
    // in-between two entries.
    var actualMax = maxLength + separatorSpace;
    // Check if at least three entries fit next to each other and prevent grouping
    // of arrays that contains entries of very different length (i.e., if a single
    // entry is longer than 1/5 of all other entries combined). Otherwise the
    // space in-between small entries would be enormous.
    if (actualMax * 3 + ctx.indentationLvl < ctx.breakLength &&
        (totalLength / actualMax > 5 || maxLength <= 6)) {
        var approxCharHeights = 2.5;
        var averageBias = Math.sqrt(actualMax - totalLength / output.length);
        var biasedMax = Math.max(actualMax - 3 - averageBias, 1);
        // Dynamically check how many columns seem possible.
        var columns = Math.min(
        // Ideally a square should be drawn. We expect a character to be about 2.5
        // times as high as wide. This is the area formula to calculate a square
        // which contains n rectangles of size `actualMax * approxCharHeights`.
        // Divide that by `actualMax` to receive the correct number of columns.
        // The added bias increases the columns for short entries.
        Math.round(Math.sqrt(approxCharHeights * biasedMax * outputLength) / biasedMax), 
        // Do not exceed the breakLength.
        Math.floor((ctx.breakLength - ctx.indentationLvl) / actualMax), 
        // Limit array grouping for small `compact` modes as the user requested
        // minimal grouping.
        ctx.compact * 4, 
        // Limit the columns to a maximum of fifteen.
        15);
        // Return with the original output if no grouping should happen.
        if (columns <= 1) {
            return output;
        }
        var tmp = [];
        var maxLineLength = [];
        for (var i_1 = 0; i_1 < columns; i_1++) {
            var lineMaxLength = 0;
            for (var j = i_1; j < output.length; j += columns) {
                if (dataLen[j] > lineMaxLength) {
                    lineMaxLength = dataLen[j];
                }
            }
            lineMaxLength += separatorSpace;
            maxLineLength[i_1] = lineMaxLength;
        }
        var order = String.prototype.padStart;
        if (value !== undefined) {
            for (var i_2 = 0; i_2 < output.length; i_2++) {
                if (typeof value[i_2] !== "number" && typeof value[i_2] !== "bigint") {
                    order = String.prototype.padEnd;
                    break;
                }
            }
        }
        // Each iteration creates a single line of grouped entries.
        for (var i_3 = 0; i_3 < outputLength; i_3 += columns) {
            // The last lines may contain less entries than columns.
            var max = Math.min(i_3 + columns, outputLength);
            var str = "";
            var j = i_3;
            for (; j < max - 1; j++) {
                // Calculate extra color padding in case it's active. This has to be
                // done line by line as some lines might contain more colors than
                // others.
                var padding = maxLineLength[j - i_3] + output[j].length - dataLen[j];
                str += order.call("".concat(output[j], ", "), padding, " ");
            }
            if (order === String.prototype.padStart) {
                var padding = maxLineLength[j - i_3] + output[j].length - dataLen[j] - separatorSpace;
                str += String.prototype.padStart.call(output[j], padding, " ");
            }
            else {
                str += output[j];
            }
            tmp.push(str);
        }
        if (100 < output.length) {
            tmp.push(output[outputLength]);
        }
        output = tmp;
    }
    return output;
}
var stripCommentsRegExp = new RegExp("(\\/\\/.*?\\n)|(\\/\\*(.|\\n)*?\\*\\/)", "g");
var classRegExp = new RegExp("^(\\s+[^(]*?)\\s*{");
function getFunctionBase(value, constructor, tag) {
    var stringified = value.toString();
    if (stringified.startsWith("class") && stringified.endsWith("}")) {
        var slice = stringified.slice(5, -1);
        var bracketIndex = slice.indexOf("{");
        if (bracketIndex !== -1 &&
            (!slice.slice(0, bracketIndex).includes("(") ||
                // Slow path to guarantee that it's indeed a class.
                classRegExp.exec(
                // @ts-expect-error this is fine
                RegExp.prototype[Symbol.replace].call(stripCommentsRegExp, slice, "")) !== null)) {
            return getClassBase(value, constructor, tag);
        }
    }
    var type = "Function";
    if (isGeneratorFunction(value)) {
        type = "Generator".concat(type);
    }
    if (isAsyncFunction(value)) {
        type = "Async".concat(type);
    }
    var base = "[".concat(type);
    if (constructor === null) {
        base += " (null prototype)";
    }
    if (value.name === "") {
        base += " (anonymous)";
    }
    else {
        base += ": ".concat(value.name);
    }
    base += "]";
    if (constructor !== type && constructor !== null) {
        base += " ".concat(constructor);
    }
    if (tag !== "" && constructor !== tag) {
        base += " [".concat(tag, "]");
    }
    return base;
}
function getClassBase(value, constructor, tag) {
    function hasName(value) {
        return Object.hasOwn(value, "name");
    }
    var name = (hasName(value) && value.name) || "(anonymous)";
    var base = "class ".concat(name);
    if (constructor !== "Function" && constructor !== null) {
        base += " [".concat(constructor, "]");
    }
    if (tag !== "" && constructor !== tag) {
        base += " [".concat(tag, "]");
    }
    if (constructor !== null) {
        var superName = Object.getPrototypeOf(value).name;
        if (superName) {
            base += " extends ".concat(superName);
        }
    }
    else {
        base += " extends [null prototype]";
    }
    return "[".concat(base, "]");
}
function addPrototypeProperties(ctx, main, obj, recurseTimes, output) {
    var depth = 0;
    var keys = [];
    var keySet;
    do {
        if (depth !== 0 || main === obj) {
            obj = Object.getPrototypeOf(obj);
            // Stop as soon as a null prototype is encountered.
            if (obj === null) {
                return;
            }
            // Stop as soon as a built-in object type is detected.
            var descriptor = Object.getOwnPropertyDescriptor(obj, "constructor");
            if (descriptor !== undefined &&
                typeof descriptor.value === "function" &&
                builtInObjects.has(descriptor.value.name)) {
                return;
            }
        }
        if (depth === 0) {
            keySet = new Set();
        }
        else {
            keys.forEach(function (key) { return keySet.add(key); });
        }
        // Get all own property names and symbols.
        keys = Reflect.ownKeys(obj);
        ctx.seen.push(main);
        for (var _i = 0, keys_1 = keys; _i < keys_1.length; _i++) {
            var key = keys_1[_i];
            // Ignore the `constructor` property and keys that exist on layers above.
            if (key === "constructor" ||
                Object.hasOwn(main, key) ||
                // @ts-expect-error this is fine
                (depth !== 0 && keySet.has(key))) {
                continue;
            }
            var desc = Object.getOwnPropertyDescriptor(obj, key);
            if (typeof desc.value === "function") {
                continue;
            }
            var value = formatProperty(ctx, 
            // @ts-expect-error this is fine
            obj, recurseTimes, key, kObjectType, desc, main);
            output.push(value);
        }
        ctx.seen.pop();
        // Limit the inspection to up to three prototype layers. Using `recurseTimes`
        // is not a good choice here, because it's as if the properties are declared
        // on the current object from the users perspective.
    } while (++depth !== 3);
}
function formatProperty(ctx, value, recurseTimes, key, type, desc, original) {
    if (original === void 0) { original = value; }
    var name, str;
    var extra = " ";
    desc = desc ||
        Object.getOwnPropertyDescriptor(value, key) || {
        value: value[key],
        enumerable: true,
    };
    if (desc.value !== undefined) {
        var diff = ctx.compact !== true || type !== kObjectType ? 2 : 3;
        ctx.indentationLvl += diff;
        str = formatValue(ctx, desc.value, recurseTimes);
        if (diff === 3 && ctx.breakLength < getStringWidth(str, ctx.colors)) {
            extra = "\n".concat(" ".repeat(ctx.indentationLvl));
        }
        ctx.indentationLvl -= diff;
    }
    else if (desc.get !== undefined) {
        var label = desc.set !== undefined ? "Getter/Setter" : "Getter";
        var s = ctx.stylize;
        var sp = "special";
        if (ctx.getters &&
            (ctx.getters === true ||
                (ctx.getters === "get" && desc.set === undefined) ||
                (ctx.getters === "set" && desc.set !== undefined))) {
            try {
                var tmp = desc.get.call(original);
                ctx.indentationLvl += 2;
                if (tmp === null) {
                    str = "".concat(s("[".concat(label, ":"), sp), " ").concat(s("null", "null")).concat(s("]", sp));
                }
                else if (typeof tmp === "object") {
                    str = "".concat(s("[".concat(label, "]"), sp), " ").concat(formatValue(ctx, tmp, recurseTimes));
                }
                else {
                    var primitive = formatPrimitive(s, tmp, ctx);
                    str = "".concat(s("[".concat(label, ":"), sp), " ").concat(primitive).concat(s("]", sp));
                }
                ctx.indentationLvl -= 2;
            }
            catch (err) {
                var message = "<Inspection threw (".concat(err.message, ")>");
                str = "".concat(s("[".concat(label, ":"), sp), " ").concat(message).concat(s("]", sp));
            }
        }
        else {
            str = ctx.stylize("[".concat(label, "]"), sp);
        }
    }
    else if (desc.set !== undefined) {
        str = ctx.stylize("[Setter]", "special");
    }
    else {
        str = ctx.stylize("undefined", "undefined");
    }
    if (type === kArrayType) {
        return str;
    }
    if (typeof key === "symbol") {
        name = "[".concat(ctx.stylize(maybeQuoteSymbol(key, ctx), "symbol"), "]");
    }
    else if (key === "__proto__") {
        name = "['__proto__']";
    }
    else if (desc.enumerable === false) {
        var tmp = key.replace(strEscapeSequencesReplacer, escapeFn);
        name = "[".concat(tmp, "]");
    }
    else if (keyStrRegExp.test(key)) {
        name = ctx.stylize(key, "name");
    }
    else {
        name = ctx.stylize(quoteString(key, ctx), "string");
    }
    return "".concat(name, ":").concat(extra).concat(str);
}
function isInstanceof(proto, object) {
    try {
        return Object.prototype.isPrototypeOf.call(proto, object);
    }
    catch (_b) {
        return false;
    }
}
function getConstructorName(obj, ctx, recurseTimes, protoProps) {
    var firstProto;
    var tmp = obj;
    while (obj || isUndetectableObject(obj)) {
        var descriptor = void 0;
        try {
            descriptor = Object.getOwnPropertyDescriptor(obj, "constructor");
        }
        catch (_b) {
            /* this could fail */
        }
        if (descriptor !== undefined &&
            typeof descriptor.value === "function" &&
            descriptor.value.name !== "" &&
            isInstanceof(descriptor.value.prototype, tmp)) {
            if (protoProps !== undefined &&
                (firstProto !== obj || !builtInObjects.has(descriptor.value.name))) {
                addPrototypeProperties(ctx, tmp, firstProto || tmp, recurseTimes, 
                // @ts-expect-error this is fine
                protoProps);
            }
            return String(descriptor.value.name);
        }
        obj = Object.getPrototypeOf(obj);
        if (firstProto === undefined) {
            firstProto = obj;
        }
    }
    if (firstProto === null) {
        return null;
    }
    // @ts-expect-error this is fine
    var res = tmp.prototype.name;
    if (recurseTimes > ctx.depth && ctx.depth !== null) {
        return "".concat(res, " <Complex prototype>");
    }
    var protoConstr = getConstructorName(
    // @ts-expect-error this is fine
    firstProto, ctx, recurseTimes + 1, protoProps);
    if (protoConstr === null) {
        return "".concat(res, " <").concat(inspect(firstProto, __assign(__assign({}, ctx), { depth: -1, __proto__: null })), ">");
    }
    return "".concat(res, " <").concat(protoConstr, ">");
}
function inspect(value, inspectOptions) {
    if (inspectOptions === void 0) { inspectOptions = {
        __proto__: null,
    }; }
    // Default options
    var ctx = __assign(__assign({}, getDefaultInspectOptions()), inspectOptions);
    return formatValue(ctx, value, 0);
}
exports.inspect = inspect;
var countMap = new Map();
// const timerMap = new Map();
var isConsoleInstance = Symbol("isConsoleInstance");
var Console = /** @class */ (function () {
    function Console(printFunc) {
        var _b;
        var _this = this;
        _Console_printFunc.set(this, void 0);
        this.indentLevel = 0;
        this[_a] = false;
        this.log = function () {
            var args = [];
            for (var _i = 0; _i < arguments.length; _i++) {
                args[_i] = arguments[_i];
            }
            __classPrivateFieldGet(_this, _Console_printFunc, "f").call(_this, inspectArgs(args, __assign(__assign({}, getDefaultInspectOptions()), { indentLevel: _this.indentLevel, __proto__: null })) + "\n", 1);
        };
        this.debug = function () {
            var args = [];
            for (var _i = 0; _i < arguments.length; _i++) {
                args[_i] = arguments[_i];
            }
            __classPrivateFieldGet(_this, _Console_printFunc, "f").call(_this, inspectArgs(args, __assign(__assign({}, getDefaultInspectOptions()), { indentLevel: _this.indentLevel, __proto__: null })) + "\n", 0);
        };
        this.info = function () {
            var args = [];
            for (var _i = 0; _i < arguments.length; _i++) {
                args[_i] = arguments[_i];
            }
            __classPrivateFieldGet(_this, _Console_printFunc, "f").call(_this, inspectArgs(args, __assign(__assign({}, getDefaultInspectOptions()), { indentLevel: _this.indentLevel, __proto__: null })) + "\n", 1);
        };
        this.dir = function (obj, options) {
            if (obj === void 0) { obj = undefined; }
            if (options === void 0) { options = { __proto__: null }; }
            __classPrivateFieldGet(_this, _Console_printFunc, "f").call(_this, inspectArgs([obj], __assign(__assign({}, getDefaultInspectOptions()), options)) + "\n", 1);
        };
        this.dirxml = this.dir;
        this.warn = function () {
            var args = [];
            for (var _i = 0; _i < arguments.length; _i++) {
                args[_i] = arguments[_i];
            }
            __classPrivateFieldGet(_this, _Console_printFunc, "f").call(_this, inspectArgs(args, __assign(__assign({}, getDefaultInspectOptions()), { indentLevel: _this.indentLevel, __proto__: null })) + "\n", 2);
        };
        this.error = function () {
            var args = [];
            for (var _i = 0; _i < arguments.length; _i++) {
                args[_i] = arguments[_i];
            }
            __classPrivateFieldGet(_this, _Console_printFunc, "f").call(_this, inspectArgs(args, __assign(__assign({}, getDefaultInspectOptions()), { indentLevel: _this.indentLevel, __proto__: null })) + "\n", 3);
        };
        this.assert = function (condition) {
            if (condition === void 0) { condition = false; }
            var args = [];
            for (var _i = 1; _i < arguments.length; _i++) {
                args[_i - 1] = arguments[_i];
            }
            if (condition) {
                return;
            }
            if (args.length === 0) {
                _this.error("Assertion failed");
                return;
            }
            var first = args[0], rest = args.slice(1);
            if (typeof first === "string") {
                _this.error.apply(_this, __spreadArray(["Assertion failed: ".concat(first)], rest, false));
                return;
            }
            _this.error.apply(_this, __spreadArray(["Assertion failed:"], args, false));
        };
        this.count = function (label) {
            if (label === void 0) { label = "default"; }
            label = String(label);
            if (countMap.has(label)) {
                var current = countMap.get(label) || 0;
                countMap.set(label, current + 1);
            }
            else {
                countMap.set(label, 1);
            }
            _this.info("".concat(label, ": ").concat(countMap.get(label)));
        };
        this.countReset = function (label) {
            if (label === void 0) { label = "default"; }
            label = String(label);
            if (countMap.has(label)) {
                countMap.set(label, 0);
            }
            else {
                _this.warn("Count for '".concat(label, "' does not exist"));
            }
        };
        __classPrivateFieldSet(this, _Console_printFunc, printFunc, "f");
        this[isConsoleInstance] = true;
        this.indentLevel = 0;
        // ref https://console.spec.whatwg.org/#console-namespace
        // For historical web-compatibility reasons, the namespace object for
        // console must have as its [[Prototype]] an empty object, created as if
        // by ObjectCreate(%ObjectPrototype%), instead of %ObjectPrototype%.
        var console = Object.create({}, (_b = {},
            _b[Symbol.toStringTag] = {
                enumerable: false,
                writable: false,
                configurable: true,
                value: "console",
            },
            _b));
        Object.assign(console, this);
        return console;
    }
    return Console;
}());
exports.Console = Console;
_Console_printFunc = new WeakMap(), _a = isConsoleInstance;
