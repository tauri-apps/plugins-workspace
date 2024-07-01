/// <reference lib="esnext" />

function isAnyArrayBuffer(
  value: unknown,
): value is ArrayBuffer | SharedArrayBuffer {
  return value instanceof ArrayBuffer || value instanceof SharedArrayBuffer;
}

function isArgumentsObject(value: unknown): boolean {
  return (
    typeof value == "object" &&
    Object.prototype.toString.call(value) === "[object Arguments]"
  );
}
function isArrayBuffer(value: unknown): value is ArrayBuffer {
  return value instanceof ArrayBuffer;
}
function isDataView(value: unknown): value is DataView {
  return ArrayBuffer.isView(value) && value instanceof DataView;
}
function isDate(value: unknown): value is Date {
  return value instanceof Date;
}
function isMap(value: unknown): value is Map<unknown, unknown> {
  return value instanceof Map;
}
function isPromise(value: unknown): value is Promise<unknown> {
  return value instanceof Promise;
}
function isRegExp(value: unknown): value is RegExp {
  return value instanceof RegExp;
}
function isSet(value: unknown): value is Set<unknown> {
  return value instanceof Set;
}
function isGeneratorFunction(value: unknown): value is GeneratorFunction {
  return (
    typeof value === "function" &&
    // @ts-expect-error this errors idk
    value[Symbol.toStringTag] === "GeneratorFunction"
  );
}
function isAsyncFunction(
  value: unknown,
): value is (...args: unknown[]) => Promise<unknown> {
  return (
    typeof value === "function" &&
    // @ts-expect-error this errors idk
    value[Symbol.toStringTag] === "AsyncFunction"
  );
}

type TypedArray =
  | Int8Array
  | Uint8Array
  | Uint8ClampedArray
  | Int16Array
  | Uint16Array
  | Int32Array
  | Uint32Array
  | Float32Array
  | Float64Array
  | BigInt64Array
  | BigUint64Array;

function isTypedArray(value: unknown): value is TypedArray {
  return (
    value instanceof Int8Array ||
    value instanceof Uint8Array ||
    value instanceof Uint8ClampedArray ||
    value instanceof Int16Array ||
    value instanceof Uint16Array ||
    value instanceof Int32Array ||
    value instanceof Uint32Array ||
    value instanceof Float32Array ||
    value instanceof Float64Array ||
    value instanceof BigInt64Array ||
    value instanceof BigUint64Array
  );
}
function isWeakMap(value: unknown): value is WeakMap<object, unknown> {
  return value instanceof WeakMap;
}
function isWeakSet(value: unknown): value is WeakSet<object> {
  return value instanceof WeakSet;
}

const kObjectType = 0;
const kArrayType = 1;
const kArrayExtrasType = 2;

const kMinLineLength = 16;

const STR_ABBREVIATE_SIZE = 10_000;

interface InspectOptions {
  strAbbreviateSize?: number;
  trailingComma: boolean;
  indentLevel: number;
  indentationLvl: number;
  currentDepth: number;
  stylize: (str: string, flavor?: string) => string;

  showHidden: boolean;
  depth: number;
  colors: boolean;
  showProxy: boolean;
  breakLength: number;
  escapeSequences: boolean;
  compact: number | boolean;
  sorted: boolean;
  getters: boolean;

  budget: Record<string, number>;
  seen: unknown[];
  circular: Map<unknown, number>;
  quotes: string[];
}

const denoInspectDefaultOptions: Omit<
  InspectOptions,
  "budget" | "seen" | "circular" | "quotes"
> = {
  indentationLvl: 0,
  currentDepth: 0,
  stylize: (str) => str,

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

function getDefaultInspectOptions(): InspectOptions {
  return {
    budget: {},
    seen: [],
    circular: new Map(),
    quotes: [],
    ...denoInspectDefaultOptions,
  };
}

const builtInObjectsRegExp = new RegExp("^[A-Z][a-zA-Z0-9]+$");
const builtInObjects = new Set(
  Object.getOwnPropertyNames(globalThis).filter((e) =>
    builtInObjectsRegExp.test(e),
  ),
);

// https://tc39.es/ecma262/#sec-IsHTMLDDA-internal-slot
const isUndetectableObject = (v: unknown) =>
  typeof v === "undefined" && v !== undefined;

const strEscapeSequencesReplacer = new RegExp(
  "[\x00-\x1f\x27\x5c\x7f-\x9f]",
  "g",
);

const keyStrRegExp = new RegExp("^[a-zA-Z_][a-zA-Z_0-9]*$");
const numberRegExp = new RegExp("^(0|[1-9][0-9]*)$");

// Escaped control characters (plus the single quote and the backslash). Use
// empty strings to fill up unused entries.
// deno-fmt-ignore
const meta = [
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

const escapeFn = (str: string) => meta[str.charCodeAt(0)];

// Regex used for ansi escape code splitting
// Adopted from https://github.com/chalk/ansi-regex/blob/HEAD/index.js
// License: MIT, authors: @sindresorhus, Qix-, arjunmehta and LitoMore
// Matches all ansi escape code sequences in a string
const ansiPattern =
  "[\\u001B\\u009B][[\\]()#;?]*" +
  "(?:(?:(?:(?:;[-a-zA-Z\\d\\/#&.:=?%@~_]+)*" +
  "|[a-zA-Z\\d]+(?:;[-a-zA-Z\\d\\/#&.:=?%@~_]*)*)?\\u0007)" +
  "|(?:(?:\\d{1,4}(?:;\\d{0,4})*)?[\\dA-PR-TZcf-ntqry=><~]))";
const ansi = new RegExp(ansiPattern, "g");

/**
 * Remove all VT control characters. Use to estimate displayed string width.
 */
export function stripVTControlCharacters(str: string) {
  return str.replace(ansi, "");
}

export function getStringWidth(str: string, removeControlChars = true) {
  let width = 0;

  if (removeControlChars) {
    str = stripVTControlCharacters(str);
  }
  str = str.normalize("NFC");
  for (const char of str) {
    const code = char.codePointAt(0)!;
    if (isFullWidthCodePoint(code)) {
      width += 2;
    } else if (!isZeroWidthCodePoint(code)) {
      width++;
    }
  }

  return width;
}

const isZeroWidthCodePoint = (code: number) => {
  return (
    code <= 0x1f || // C0 control codes
    (code >= 0x7f && code <= 0x9f) || // C1 control codes
    (code >= 0x300 && code <= 0x36f) || // Combining Diacritical Marks
    (code >= 0x200b && code <= 0x200f) || // Modifying Invisible Characters
    // Combining Diacritical Marks for Symbols
    (code >= 0x20d0 && code <= 0x20ff) ||
    (code >= 0xfe00 && code <= 0xfe0f) || // Variation Selectors
    (code >= 0xfe20 && code <= 0xfe2f) || // Combining Half Marks
    (code >= 0xe0100 && code <= 0xe01ef)
  ); // Variation Selectors
};

function isFullWidthCodePoint(code: number) {
  // Code points are partially derived from:
  // http://www.unicode.org/Public/UNIDATA/EastAsianWidth.txt
  return (
    code >= 0x1100 &&
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
      (code >= 0x20000 && code <= 0x3fffd))
  );
}

const DEFAULT_INDENT = "  ";

function inspectArgs(
  args: unknown[],
  inspectOptions: Partial<InspectOptions> & { __proto__: null } = {
    __proto__: null,
  },
): string {
  const ctx = {
    ...getDefaultInspectOptions(),
    ...inspectOptions,
  };

  const first = args[0];
  let a = 0;
  let string = "";

  if (typeof first == "string" && args.length > 1) {
    a++;
    // Index of the first not-yet-appended character. Use this so we only
    // have to append to `string` when a substitution occurs / at the end.
    let appendedChars = 0;
    for (let i = 0; i < first.length - 1; i++) {
      if (first[i] == "%") {
        const char = first[++i];
        if (a < args.length) {
          let formattedArg = null;
          if (char == "s") {
            // Format as a string.
            formattedArg = String(args[a++]);
          } else if (["d", "i"].includes(char)) {
            // Format as an integer.
            const value = args[a++];
            if (typeof value == "bigint") {
              formattedArg = `${value}n`;
            } else if (typeof value == "number") {
              formattedArg = `${Number.parseInt(String(value))}`;
            } else {
              formattedArg = "NaN";
            }
          } else if (char == "f") {
            // Format as a floating point value.
            const value = args[a++];
            if (typeof value == "number") {
              formattedArg = `${value}`;
            } else {
              formattedArg = "NaN";
            }
          } else if (["O", "o"].includes(char)) {
            // Format as an object.
            formattedArg = formatValue(ctx, args[a++], 0);
          } else if (char == "c") {
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
      string += args[a] as string;
    } else {
      // Use default maximum depth for null or undefined arguments.
      string += formatValue(ctx, args[a], 0);
    }
  }

  if (ctx.indentLevel > 0) {
    const groupIndent = DEFAULT_INDENT.repeat(ctx.indentLevel);
    string = groupIndent + string.replaceAll("\n", `\n${groupIndent}`);
  }

  return string;
}

function formatValue(
  ctx: InspectOptions,
  value: unknown,
  recurseTimes: number,
  typedArray?: boolean,
): string {
  // Primitive types cannot have properties.
  if (
    typeof value !== "object" &&
    typeof value !== "function" &&
    !isUndetectableObject(value)
  ) {
    return formatPrimitive(ctx.stylize, value, ctx);
  }
  if (value === null) {
    return ctx.stylize("null", "null");
  }

  // Using an array here is actually better for the average case than using
  // a Set. `seen` will only check for the depth and will never grow too large.
  if (ctx.seen.includes(value)) {
    let index = 1;
    if (ctx.circular === undefined) {
      ctx.circular = new Map();
      ctx.circular.set(value, index);
    } else {
      index = ctx.circular.get(value)!;
      if (index === undefined) {
        index = ctx.circular.size + 1;
        ctx.circular.set(value, index);
      }
    }
    return ctx.stylize(`[Circular *${index}]`, "special");
  }

  return formatRaw(ctx, value as object, recurseTimes, typedArray);
}

const formatPrimitiveRegExp = new RegExp("(?<=\n)");
function formatPrimitive(
  fn: InspectOptions["stylize"],
  value: unknown,
  ctx: InspectOptions,
) {
  if (typeof value === "string") {
    if (
      // TODO(BridgeAR): Add unicode support. Use the readline getStringWidth
      // function.
      value.length > kMinLineLength &&
      value.length > ctx.breakLength - ctx.indentationLvl - 4
    ) {
      return value
        .split(formatPrimitiveRegExp)
        .map((line) => fn(quoteString(line, ctx), "string"))
        .join(` +\n${" ".repeat(ctx.indentationLvl + 2)}`);
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
    return fn(`${value}`, "boolean");
  }
  if (typeof value === "undefined") {
    return fn("undefined", "undefined");
  }
  // es6 symbol primitive
  return fn(maybeQuoteSymbol(value as symbol, ctx), "symbol");
}

function formatNumber(fn: InspectOptions["stylize"], value: number) {
  // Format -0 as '-0'. Checking `value === -0` won't distinguish 0 from -0.
  return fn(Object.is(value, -0) ? "-0" : `${value}`, "number");
}

function formatBigInt(fn: InspectOptions["stylize"], value: bigint) {
  return fn(`${value}n`, "bigint");
}

const QUOTE_SYMBOL_REG = new RegExp(/^[a-zA-Z_][a-zA-Z_.0-9]*$/);

function maybeQuoteSymbol(symbol: symbol, ctx: InspectOptions) {
  const description = symbol.description;

  if (description === undefined) {
    return symbol.toString();
  }

  if (QUOTE_SYMBOL_REG.test(description)) {
    return symbol.toString();
  }

  return `Symbol(${quoteString(description, ctx)})`;
}

/** Surround the string in quotes.
 *
 * The quote symbol is chosen by taking the first of the `QUOTES` array which
 * does not occur in the string. If they all occur, settle with `QUOTES[0]`.
 *
 * Insert a backslash before any occurrence of the chosen quote symbol and
 * before any backslash.
 */
function quoteString(string: string, ctx: InspectOptions) {
  const quote = ctx.quotes.find((c) => !string.includes(c)) ?? ctx.quotes[0];

  const escapePattern = new RegExp(`(?=[${quote}\\\\])`, "g");
  string = string.replace(escapePattern, "\\");
  if (ctx.escapeSequences) {
    string = replaceEscapeSequences(string);
  }
  return `${quote}${string}${quote}`;
}

const ESCAPE_PATTERN = new RegExp(/([\b\f\n\r\t\v])/g);
const ESCAPE_MAP: Readonly<Record<string, string>> = Object.freeze({
  "\b": "\\b",
  "\f": "\\f",
  "\n": "\\n",
  "\r": "\\r",
  "\t": "\\t",
  "\v": "\\v",
});

const ESCAPE_PATTERN2 = new RegExp("[\x00-\x1f\x7f-\x9f]", "g");

// Replace escape sequences that can modify output.
function replaceEscapeSequences(string: string) {
  return string
    .replace(ESCAPE_PATTERN, (c) => ESCAPE_MAP[c])
    .replace(
      ESCAPE_PATTERN2,
      (c) => "\\x" + c.charCodeAt(0).toString(16).padStart(2, "0"),
    );
}

function formatSet(
  value: Set<unknown>,
  ctx: InspectOptions,
  _ignored: any,
  recurseTimes: number,
) {
  ctx.indentationLvl += 2;

  const values = [...value];
  const valLen = value.size;
  const len = Math.min(100, valLen);

  const remaining = valLen - len;
  const output = [];
  for (let i = 0; i < len; i++) {
    output.push(formatValue(ctx, values[i], recurseTimes));
  }
  if (remaining > 0) {
    output.push(`... ${remaining} more item${remaining > 1 ? "s" : ""}`);
  }

  ctx.indentationLvl -= 2;
  return output;
}

function formatMap(
  value: Map<unknown, unknown>,
  ctx: InspectOptions,
  _ignored: any,
  recurseTimes: number,
) {
  ctx.indentationLvl += 2;

  const values = [...value];
  const valLen = value.size;
  const len = Math.min(100, valLen);

  const remaining = valLen - len;
  const output = [];
  for (let i = 0; i < len; i++) {
    output.push(
      `${formatValue(ctx, values[i][0], recurseTimes)} => ${formatValue(
        ctx,
        values[i][1],
        recurseTimes,
      )}`,
    );
  }
  if (remaining > 0) {
    output.push(`... ${remaining} more item${remaining > 1 ? "s" : ""}`);
  }

  ctx.indentationLvl -= 2;
  return output;
}

function formatArray(
  ctx: InspectOptions,
  value: Array<unknown>,
  recurseTimes: number,
): string[] {
  const valLen = value.length;
  const len = Math.min(100, valLen);

  const remaining = valLen - len;
  const output: string[] = [];
  for (let i = 0; i < len; i++) {
    // Special handle sparse arrays.
    if (!Object.hasOwn(value, i)) {
      return formatSpecialArray(ctx, value, recurseTimes, len, output, i);
    }
    // @ts-expect-error this is fine
    output.push(formatProperty(ctx, value, recurseTimes, i, kArrayType));
  }
  if (remaining > 0) {
    output.push(`... ${remaining} more item${remaining > 1 ? "s" : ""}`);
  }
  return output;
}

// The array is sparse and/or has extra keys
function formatSpecialArray(
  ctx: InspectOptions,
  value: Array<unknown>,
  recurseTimes: number,
  maxLength: number,
  output: string[],
  i: number,
) {
  const keys = Object.keys(value);
  let index = i;
  for (; i < keys.length && output.length < maxLength; i++) {
    const key = keys[i];
    const tmp = +key;
    // Arrays can only have up to 2^32 - 1 entries
    if (tmp > 2 ** 32 - 2) {
      break;
    }
    if (`${index}` !== key) {
      if (!numberRegExp.test(key)) {
        break;
      }
      const emptyItems = tmp - index;
      const ending = emptyItems > 1 ? "s" : "";
      const message = `<${emptyItems} empty item${ending}>`;
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
  const remaining = value.length - index;
  if (output.length !== maxLength) {
    if (remaining > 0) {
      const ending = remaining > 1 ? "s" : "";
      const message = `<${remaining} empty item${ending}>`;
      output.push(ctx.stylize(message, "undefined"));
    }
  } else if (remaining > 0) {
    output.push(`... ${remaining} more item${remaining > 1 ? "s" : ""}`);
  }
  return output;
}

function formatTypedArray(
  value: TypedArray,
  length: number,
  ctx: InspectOptions,
  _ignored: any,
  recurseTimes: number,
) {
  const maxLength = Math.min(100, length);
  const remaining = value.length - maxLength;
  const output = [];

  // @ts-expect-error this errors idk
  const elementFormatter: (
    stylize: (a: string, b: string) => string,
    value: number | bigint,
  ) => string =
    value.length > 0 && typeof value[0] === "number"
      ? formatNumber
      : formatBigInt;
  for (let i = 0; i < maxLength; ++i) {
    output[i] = elementFormatter(ctx.stylize, value[i]);
  }
  if (remaining > 0) {
    output[maxLength] = `... ${remaining} more item${remaining > 1 ? "s" : ""}`;
  }
  if (ctx.showHidden) {
    // .buffer goes last, it's not a primitive like the others.
    // All besides `BYTES_PER_ELEMENT` are actually getters.
    ctx.indentationLvl += 2;
    for (const key of [
      "BYTES_PER_ELEMENT",
      "length",
      "byteLength",
      "byteOffset",
      "buffer",
    ]) {
      const str = formatValue(
        ctx,
        (value as unknown as Record<string, unknown>)[key],
        recurseTimes,
        true,
      );
      output.push(`[${key}]: ${str}`);
    }
    ctx.indentationLvl -= 2;
  }
  return output;
}

const arrayBufferRegExp = new RegExp("(.{2})", "g");
function formatArrayBuffer(
  ctx: InspectOptions,
  value: ArrayBuffer | SharedArrayBuffer,
  _ignored: number,
) {
  let valLen: number;
  try {
    valLen = value.byteLength;
  } catch {
    valLen = getSharedArrayBufferByteLength(value as SharedArrayBuffer);
  }
  const len = Math.min(100, valLen);
  let buffer;
  try {
    buffer = new Uint8Array(value, 0, len);
  } catch {
    return [ctx.stylize("(detached)", "special")];
  }
  let str = hexSlice(buffer).replace(arrayBufferRegExp, "$1 ").trim();

  const remaining = valLen - len;
  if (remaining > 0) {
    str += ` ... ${remaining} more byte${remaining > 1 ? "s" : ""}`;
  }
  return [`${ctx.stylize("[Uint8Contents]", "special")}: <${str}>`];
}

function formatPromise(
  ctx: InspectOptions,
  value: Promise<unknown>,
  recurseTimes: number,
): string[] {
  return ["Promise"];
}

function formatWeakCollection(ctx: InspectOptions) {
  return [ctx.stylize("<items unknown>", "special")];
}

function formatWeakSet(
  ctx: InspectOptions,
  value: WeakSet<object>,
  recurseTimes: number,
): string[] {
  return ["WeakSet"];
}

function formatWeakMap(
  ctx: InspectOptions,
  value: WeakMap<object, unknown>,
  recurseTimes: number,
): string[] {
  return ["WeakMap"];
}

const hexSliceLookupTable = (function () {
  const alphabet = "0123456789abcdef";
  const table = [];
  for (let i = 0; i < 16; ++i) {
    const i16 = i * 16;
    for (let j = 0; j < 16; ++j) {
      table[i16 + j] = alphabet[i] + alphabet[j];
    }
  }
  return table;
})();

function hexSlice(buf: Uint8Array, start?: number, end?: number) {
  const len = buf.length;
  if (!start || start < 0) {
    start = 0;
  }
  if (!end || end < 0 || end > len) {
    end = len;
  }
  let out = "";
  for (let i = start; i < end; ++i) {
    out += hexSliceLookupTable[buf[i]];
  }
  return out;
}

// https://tc39.es/ecma262/#sec-get-sharedarraybuffer.prototype.bytelength
let _getSharedArrayBufferByteLength;

function getSharedArrayBufferByteLength(value: SharedArrayBuffer): number {
  // TODO(kt3k): add SharedArrayBuffer to primordials
  // @ts-expect-error this is fine
  _getSharedArrayBufferByteLength ??= Object.getOwnPropertyDescriptor(
    SharedArrayBuffer.prototype,
    "byteLength",
  ).get;

  return _getSharedArrayBufferByteLength.call(value);
}

// Look up the keys of the object.
function getKeys(value: object, showHidden: boolean) {
  let keys: PropertyKey[];
  const symbols = Object.getOwnPropertySymbols(value);
  if (showHidden) {
    keys = Object.getOwnPropertyNames(value);
    if (symbols.length !== 0) {
      keys.push(...symbols);
    }
  } else {
    // This might throw if `value` is a Module Namespace Object from an
    // unevaluated module, but we don't want to perform the actual type
    // check because it's expensive.
    // TODO(devsnek): track https://github.com/tc39/ecma262/issues/1209
    // and modify this logic as needed.
    try {
      keys = Object.keys(value);
    } catch (err) {
      // assert(
      //   isNativeError(err) &&
      //     err.name === "ReferenceError" &&
      //     isModuleNamespaceObject(value),
      // );
      keys = Object.getOwnPropertyNames(value);
    }
    if (symbols.length !== 0) {
      const filter = (key: PropertyKey) =>
        Object.prototype.propertyIsEnumerable.call(value, key);
      keys.push(...symbols.filter(filter));
    }
  }
  return keys;
}

function getPrefix(
  constructor: string | null,
  tag: string,
  fallback: string,
  size = "",
) {
  if (constructor === null) {
    if (tag !== "" && fallback !== tag) {
      return `[${fallback}${size}: null prototype] [${tag}] `;
    }
    return `[${fallback}${size}: null prototype] `;
  }

  if (tag !== "" && constructor !== tag) {
    return `${constructor}${size} [${tag}] `;
  }
  return `${constructor}${size} `;
}

function getCtxStyle(value: unknown, constructor: string | null, tag: string) {
  let fallback = "";
  if (constructor === null) {
    if (fallback === tag) {
      fallback = "Object";
    }
  }
  return getPrefix(constructor, tag, fallback);
}

function formatRaw(
  ctx: InspectOptions,
  value: object,
  recurseTimes: number,
  typedArray?: boolean,
): string {
  let keys: PropertyKey[] = [];
  let protoProps: string[] | undefined;
  if (ctx.showHidden && (recurseTimes <= ctx.depth || ctx.depth === null)) {
    protoProps = [];
  }

  const constructor = getConstructorName(value, ctx, recurseTimes, protoProps);
  // Reset the variable to check for this later on.
  if (protoProps !== undefined && protoProps.length === 0) {
    protoProps = undefined;
  }

  // @ts-expect-error this is fine
  let _tag: unknown = value[Symbol.toStringTag];
  // Only list the tag in case it's non-enumerable / not an own property.
  // Otherwise we'd print this twice.
  if (typeof _tag !== "string") {
    _tag = "";
  }
  const tag: string = _tag as string;
  let base = "";
  let formatter: (
    ctx: InspectOptions,
    value: any,
    recurseTimes: number,
  ) => string[] = () => [];
  let braces: string[] = [];
  let noIterator = true;
  let i = 0;

  let extrasType = kObjectType;

  // Iterators and the rest are split to reduce checks.
  // We have to check all values in case the constructor is set to null.
  // Otherwise it would not possible to identify all types properly.
  if (Reflect.has(value, Symbol.iterator) || constructor === null) {
    noIterator = false;
    if (Array.isArray(value)) {
      // Only set the constructor for non ordinary ("Array [...]") arrays.
      const prefix: string =
        constructor !== "Array" || tag !== ""
          ? getPrefix(constructor, tag, "Array", `(${value.length})`)
          : "";
      keys = Object.getOwnPropertyNames(value);
      braces = [`${prefix}[`, "]"];
      if (value.length === 0 && keys.length === 0 && protoProps === undefined) {
        return `${braces[0]}]`;
      }
      extrasType = kArrayExtrasType;
      formatter = formatArray;
    } else if (isSet(value)) {
      const size = value.size;
      const prefix = getPrefix(constructor, tag, "Set", `(${size})`);
      keys = getKeys(value, ctx.showHidden);
      formatter =
        constructor !== null
          ? formatSet.bind(null, value)
          : formatSet.bind(null, new Set(value.values()));
      if (size === 0 && keys.length === 0 && protoProps === undefined) {
        return `${prefix}{}`;
      }
      braces = [`${prefix}{`, "}"];
    } else if (isMap(value)) {
      const size = value.size;
      const prefix = getPrefix(constructor, tag, "Map", `(${size})`);
      keys = getKeys(value, ctx.showHidden);
      formatter =
        constructor !== null
          ? formatMap.bind(null, value)
          : formatMap.bind(null, new Map(value.entries()));
      if (size === 0 && keys.length === 0 && protoProps === undefined) {
        return `${prefix}{}`;
      }
      braces = [`${prefix}{`, "}"];
    } else if (isTypedArray(value)) {
      keys = Object.getOwnPropertyNames(value);
      const fallback = "";
      const size = value.length;
      const prefix = getPrefix(constructor, tag, fallback, `(${size})`);
      braces = [`${prefix}[`, "]"];
      if (value.length === 0 && keys.length === 0 && !ctx.showHidden) {
        return `${braces[0]}]`;
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
    }*/ else {
      noIterator = true;
    }
  }
  if (noIterator) {
    keys = getKeys(value, ctx.showHidden);
    braces = ["{", "}"];
    if (constructor === "Object") {
      if (isArgumentsObject(value)) {
        braces[0] = "[Arguments] {";
      } else if (tag !== "") {
        braces[0] = `${getPrefix(constructor, tag, "Object")}{`;
      }
      if (keys.length === 0 && protoProps === undefined) {
        return `${braces[0]}}`;
      }
    } else if (typeof value === "function") {
      base = getFunctionBase(value, constructor, tag);
      if (keys.length === 0 && protoProps === undefined) {
        return ctx.stylize(base, "special");
      }
    } else if (isRegExp(value)) {
      // Make RegExps say that they are RegExps
      base = (constructor !== null ? value : new RegExp(value)).toString();
      const prefix = getPrefix(constructor, tag, "RegExp");
      if (prefix !== "RegExp ") {
        base = `${prefix}${base}`;
      }
      if (
        (keys.length === 0 && protoProps === undefined) ||
        (recurseTimes > ctx.depth && ctx.depth !== null)
      ) {
        return ctx.stylize(base, "regexp");
      }
    } else if (isDate(value)) {
      if (Number.isNaN(value.getTime())) {
        return ctx.stylize("Invalid Date", "date");
      } else {
        base = value.toISOString();
        if (keys.length === 0 && protoProps === undefined) {
          return ctx.stylize(base, "date");
        }
      }
    } else if (
      // @ts-expect-error this is fine
      typeof globalThis.Temporal !== "undefined" &&
      (Object.prototype.isPrototypeOf.call(
        // @ts-expect-error this is fine
        globalThis.Temporal.Instant.prototype,
        value,
      ) ||
        Object.prototype.isPrototypeOf.call(
          // @ts-expect-error this is fine
          globalThis.Temporal.ZonedDateTime.prototype,
          value,
        ) ||
        Object.prototype.isPrototypeOf.call(
          // @ts-expect-error this is fine
          globalThis.Temporal.PlainDate.prototype,
          value,
        ) ||
        Object.prototype.isPrototypeOf.call(
          // @ts-expect-error this is fine
          globalThis.Temporal.PlainTime.prototype,
          value,
        ) ||
        Object.prototype.isPrototypeOf.call(
          // @ts-expect-error this is fine
          globalThis.Temporal.PlainDateTime.prototype,
          value,
        ) ||
        Object.prototype.isPrototypeOf.call(
          // @ts-expect-error this is fine
          globalThis.Temporal.PlainYearMonth.prototype,
          value,
        ) ||
        Object.prototype.isPrototypeOf.call(
          // @ts-expect-error this is fine
          globalThis.Temporal.PlainMonthDay.prototype,
          value,
        ) ||
        Object.prototype.isPrototypeOf.call(
          // @ts-expect-error this is fine
          globalThis.Temporal.Duration.prototype,
          value,
        ) ||
        Object.prototype.isPrototypeOf.call(
          // @ts-expect-error this is fine
          globalThis.Temporal.TimeZone.prototype,
          value,
        ) ||
        Object.prototype.isPrototypeOf.call(
          // @ts-expect-error this is fine
          globalThis.Temporal.Calendar.prototype,
          value,
        ))
    ) {
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
    } */ else if (isAnyArrayBuffer(value)) {
      // Fast path for ArrayBuffer and SharedArrayBuffer.
      // Can't do the same for DataView because it has a non-primitive
      // .buffer property that we need to recurse for.
      const arrayType = isArrayBuffer(value)
        ? "ArrayBuffer"
        : "SharedArrayBuffer";

      const prefix = getPrefix(constructor, tag, arrayType);
      if (typedArray === undefined) {
        formatter = formatArrayBuffer;
      } else if (keys.length === 0 && protoProps === undefined) {
        return (
          prefix +
          `{ byteLength: ${formatNumber(ctx.stylize, value.byteLength)} }`
        );
      }
      braces[0] = `${prefix}{`;
      keys.unshift("byteLength");
    } else if (isDataView(value)) {
      braces[0] = `${getPrefix(constructor, tag, "DataView")}{`;
      // .buffer goes last, it's not a primitive like the others.
      keys.unshift("byteLength", "byteOffset", "buffer");
    } else if (isPromise(value)) {
      braces[0] = `${getPrefix(constructor, tag, "Promise")}{`;
      formatter = formatPromise;
    } else if (isWeakSet(value)) {
      braces[0] = `${getPrefix(constructor, tag, "WeakSet")}{`;
      formatter = ctx.showHidden ? formatWeakSet : formatWeakCollection;
    } else if (isWeakMap(value)) {
      braces[0] = `${getPrefix(constructor, tag, "WeakMap")}{`;
      formatter = ctx.showHidden ? formatWeakMap : formatWeakCollection;
    } /* else if (isModuleNamespaceObject(value)) {
      braces[0] = `${getPrefix(constructor, tag, "Module")}{`;
      // Special handle keys for namespace objects.
      formatter = FunctionPrototypeBind(formatNamespaceObject, null, keys);
    } */ else {
      if (keys.length === 0 && protoProps === undefined) {
        // TODO(wafuwafu13): Implement
        // if (isExternal(value)) {
        //   const address = getExternalValue(value).toString(16);
        //   return ctx.stylize(`[External: ${address}]`, 'special');
        // }
        return `${getCtxStyle(value, constructor, tag)}{}`;
      }
      braces[0] = `${getCtxStyle(value, constructor, tag)}{`;
    }
  }

  if (recurseTimes > ctx.depth && ctx.depth !== null) {
    let constructorName = getCtxStyle(value, constructor, tag).slice(0, -1);
    if (constructor !== null) {
      constructorName = `[${constructorName}]`;
    }
    return ctx.stylize(constructorName, "special");
  }
  recurseTimes += 1;

  ctx.seen.push(value);
  ctx.currentDepth = recurseTimes;
  let output;
  try {
    output = formatter(ctx, value, recurseTimes);
    for (i = 0; i < keys.length; i++) {
      output.push(
        // @ts-expect-error this is fine
        formatProperty(ctx, value, recurseTimes, keys[i], extrasType),
      );
    }
    if (protoProps !== undefined) {
      output.push(...protoProps);
    }
  } catch (error: unknown) {
    // TODO(wafuwafu13): Implement stack overflow check
    return ctx.stylize(
      `[Internal Formatting Error] ${(error as Error).stack}`,
      "internalError",
    );
  }

  if (ctx.circular !== undefined) {
    const index = ctx.circular.get(value);
    if (index !== undefined) {
      const reference = ctx.stylize(`<ref *${index}>`, "special");
      // Add reference always to the very beginning of the output.
      if (ctx.compact !== true) {
        base = base === "" ? reference : `${reference} ${base}`;
      } else {
        braces[0] = `${reference} ${braces[0]}`;
      }
    }
  }
  ctx.seen.pop();

  if (ctx.sorted) {
    const comparator = ctx.sorted === true ? undefined : ctx.sorted;
    if (extrasType === kObjectType) {
      output = output.sort(comparator);
    } else if (keys.length > 1) {
      const sorted = output.slice(output.length - keys.length).sort(comparator);
      output.splice(output.length - keys.length, keys.length, ...sorted);
    }
  }

  const res = reduceToSingleString(
    ctx,
    output,
    base,
    braces,
    extrasType,
    recurseTimes,
    [value],
  );
  const budget = ctx.budget[ctx.indentationLvl] || 0;
  const newLength = budget + res.length;
  ctx.budget[ctx.indentationLvl] = newLength;
  // If any indentationLvl exceeds this limit, limit further inspecting to the
  // minimum. Otherwise the recursive algorithm might continue inspecting the
  // object even though the maximum string size (~2 ** 28 on 32 bit systems and
  // ~2 ** 30 on 64 bit systems) exceeded. The actual output is not limited at
  // exactly 2 ** 27 but a bit higher. This depends on the object shape.
  // This limit also makes sure that huge objects don't block the event loop
  // significantly.
  if (newLength > 2 ** 27) {
    ctx.depth = -1;
  }
  return res;
}

function reduceToSingleString(
  ctx: InspectOptions,
  output: string[],
  base: string,
  braces: string[],
  extrasType: number,
  recurseTimes: number,
  value: unknown[],
) {
  if (ctx.compact !== true) {
    if (typeof ctx.compact === "number" && ctx.compact >= 1) {
      // Memorize the original output length. In case the output is grouped,
      // prevent lining up the entries on a single line.
      const entries = output.length;
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
      if (
        ctx.currentDepth - recurseTimes < ctx.compact &&
        entries === output.length
      ) {
        // Line up all entries on a single line in case the entries do not
        // exceed `breakLength`. Add 10 as constant to start next to all other
        // factors that may reduce `breakLength`.
        const start =
          output.length +
          ctx.indentationLvl +
          braces[0].length +
          base.length +
          10;
        if (isBelowBreakLength(ctx, output, start, base)) {
          const joinedOutput = output.join(", ");
          if (!joinedOutput.includes("\n")) {
            return (
              `${base ? `${base} ` : ""}${braces[0]} ${joinedOutput}` +
              ` ${braces[1]}`
            );
          }
        }
      }
    }
    // Line up each entry on an individual line.
    const indentation = `\n${" ".repeat(ctx.indentationLvl)}`;
    return (
      `${base ? `${base} ` : ""}${braces[0]}${indentation}  ` +
      `${output.join(`,${indentation}  `)}${
        ctx.trailingComma ? "," : ""
      }${indentation}${braces[1]}`
    );
  }
  // Line up all entries on a single line in case the entries do not exceed
  // `breakLength`.
  if (isBelowBreakLength(ctx, output, 0, base)) {
    return (
      `${braces[0]}${base ? ` ${base}` : ""} ${output.join(", ")} ` + braces[1]
    );
  }
  const indentation = " ".repeat(ctx.indentationLvl);
  // If the opening "brace" is too large, like in the case of "Set {",
  // we need to force the first item to be on the next line or the
  // items will not line up correctly.
  const ln =
    base === "" && braces[0].length === 1
      ? " "
      : `${base ? ` ${base}` : ""}\n${indentation}  `;
  // Line up each entry on an individual line.
  return `${braces[0]}${ln}${output.join(`,\n${indentation}  `)} ${braces[1]}`;
}

function isBelowBreakLength(
  ctx: InspectOptions,
  output: string[],
  start: number,
  base: string,
) {
  // Each entry is separated by at least a comma. Thus, we start with a total
  // length of at least `output.length`. In addition, some cases have a
  // whitespace in-between each other that is added to the total as well.
  // TODO(BridgeAR): Add unicode support. Use the readline getStringWidth
  // function. Check the performance overhead and make it an opt-in in case it's
  // significant.
  let totalLength = output.length + start;
  if (totalLength + output.length > ctx.breakLength) {
    return false;
  }
  for (let i = 0; i < output.length; i++) {
    totalLength += output[i].length;
    if (totalLength > ctx.breakLength) {
      return false;
    }
  }
  // Do not line up properties on the same line if `base` contains line breaks.
  return base === "" || !base.includes("\n");
}

function groupArrayElements(
  ctx: InspectOptions,
  output: string[],
  value: Array<unknown>,
) {
  let totalLength = 0;
  let maxLength = 0;
  let i = 0;
  let outputLength = output.length;
  if (100 < output.length) {
    // This makes sure the "... n more items" part is not taken into account.
    outputLength--;
  }
  const separatorSpace = 2; // Add 1 for the space and 1 for the separator.
  const dataLen = [];
  // Calculate the total length of all output entries and the individual max
  // entries length of all output entries. We have to remove colors first,
  // otherwise the length would not be calculated properly.
  for (; i < outputLength; i++) {
    const len = getStringWidth(output[i], ctx.colors);
    dataLen[i] = len;
    totalLength += len + separatorSpace;
    if (maxLength < len) {
      maxLength = len;
    }
  }
  // Add two to `maxLength` as we add a single whitespace character plus a comma
  // in-between two entries.
  const actualMax = maxLength + separatorSpace;
  // Check if at least three entries fit next to each other and prevent grouping
  // of arrays that contains entries of very different length (i.e., if a single
  // entry is longer than 1/5 of all other entries combined). Otherwise the
  // space in-between small entries would be enormous.
  if (
    actualMax * 3 + ctx.indentationLvl < ctx.breakLength &&
    (totalLength / actualMax > 5 || maxLength <= 6)
  ) {
    const approxCharHeights = 2.5;
    const averageBias = Math.sqrt(actualMax - totalLength / output.length);
    const biasedMax = Math.max(actualMax - 3 - averageBias, 1);
    // Dynamically check how many columns seem possible.
    const columns = Math.min(
      // Ideally a square should be drawn. We expect a character to be about 2.5
      // times as high as wide. This is the area formula to calculate a square
      // which contains n rectangles of size `actualMax * approxCharHeights`.
      // Divide that by `actualMax` to receive the correct number of columns.
      // The added bias increases the columns for short entries.
      Math.round(
        Math.sqrt(approxCharHeights * biasedMax * outputLength) / biasedMax,
      ),
      // Do not exceed the breakLength.
      Math.floor((ctx.breakLength - ctx.indentationLvl) / actualMax),
      // Limit array grouping for small `compact` modes as the user requested
      // minimal grouping.
      (ctx.compact as number) * 4,
      // Limit the columns to a maximum of fifteen.
      15,
    );
    // Return with the original output if no grouping should happen.
    if (columns <= 1) {
      return output;
    }
    const tmp = [];
    const maxLineLength = [];
    for (let i = 0; i < columns; i++) {
      let lineMaxLength = 0;
      for (let j = i; j < output.length; j += columns) {
        if (dataLen[j] > lineMaxLength) {
          lineMaxLength = dataLen[j];
        }
      }
      lineMaxLength += separatorSpace;
      maxLineLength[i] = lineMaxLength;
    }
    let order = String.prototype.padStart;
    if (value !== undefined) {
      for (let i = 0; i < output.length; i++) {
        if (typeof value[i] !== "number" && typeof value[i] !== "bigint") {
          order = String.prototype.padEnd;
          break;
        }
      }
    }
    // Each iteration creates a single line of grouped entries.
    for (let i = 0; i < outputLength; i += columns) {
      // The last lines may contain less entries than columns.
      const max = Math.min(i + columns, outputLength);
      let str = "";
      let j = i;
      for (; j < max - 1; j++) {
        // Calculate extra color padding in case it's active. This has to be
        // done line by line as some lines might contain more colors than
        // others.
        const padding = maxLineLength[j - i] + output[j].length - dataLen[j];
        str += order.call(`${output[j]}, `, padding, " ");
      }
      if (order === String.prototype.padStart) {
        const padding =
          maxLineLength[j - i] + output[j].length - dataLen[j] - separatorSpace;
        str += String.prototype.padStart.call(output[j], padding, " ");
      } else {
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

const stripCommentsRegExp = new RegExp(
  "(\\/\\/.*?\\n)|(\\/\\*(.|\\n)*?\\*\\/)",
  "g",
);
const classRegExp = new RegExp("^(\\s+[^(]*?)\\s*{");

function getFunctionBase(
  value: CallableFunction,
  constructor: string | null,
  tag: string,
) {
  const stringified = value.toString();
  if (stringified.startsWith("class") && stringified.endsWith("}")) {
    const slice = stringified.slice(5, -1);
    const bracketIndex = slice.indexOf("{");
    if (
      bracketIndex !== -1 &&
      (!slice.slice(0, bracketIndex).includes("(") ||
        // Slow path to guarantee that it's indeed a class.
        classRegExp.exec(
          // @ts-expect-error this is fine
          RegExp.prototype[Symbol.replace].call(stripCommentsRegExp, slice, ""),
        ) !== null)
    ) {
      return getClassBase(value, constructor, tag);
    }
  }
  let type = "Function";
  if (isGeneratorFunction(value)) {
    type = `Generator${type}`;
  }
  if (isAsyncFunction(value)) {
    type = `Async${type}`;
  }
  let base = `[${type}`;
  if (constructor === null) {
    base += " (null prototype)";
  }
  if (value.name === "") {
    base += " (anonymous)";
  } else {
    base += `: ${value.name}`;
  }
  base += "]";
  if (constructor !== type && constructor !== null) {
    base += ` ${constructor}`;
  }
  if (tag !== "" && constructor !== tag) {
    base += ` [${tag}]`;
  }
  return base;
}

function getClassBase(
  value: object,
  constructor: string | null,
  tag: string,
): string {
  function hasName(value: object): value is { name: string } {
    return Object.hasOwn(value, "name");
  }
  const name = (hasName(value) && value.name) || "(anonymous)";
  let base = `class ${name}`;
  if (constructor !== "Function" && constructor !== null) {
    base += ` [${constructor}]`;
  }
  if (tag !== "" && constructor !== tag) {
    base += ` [${tag}]`;
  }
  if (constructor !== null) {
    const superName = Object.getPrototypeOf(value).name;
    if (superName) {
      base += ` extends ${superName}`;
    }
  } else {
    base += " extends [null prototype]";
  }
  return `[${base}]`;
}

function addPrototypeProperties(
  ctx: InspectOptions,
  main: object,
  obj: object,
  recurseTimes: number,
  output: string[],
) {
  let depth = 0;
  let keys: PropertyKey[] = [];
  let keySet: Set<PropertyKey>;
  do {
    if (depth !== 0 || main === obj) {
      obj = Object.getPrototypeOf(obj);
      // Stop as soon as a null prototype is encountered.
      if (obj === null) {
        return;
      }
      // Stop as soon as a built-in object type is detected.
      const descriptor = Object.getOwnPropertyDescriptor(obj, "constructor");
      if (
        descriptor !== undefined &&
        typeof descriptor.value === "function" &&
        builtInObjects.has((descriptor.value as VoidFunction).name)
      ) {
        return;
      }
    }

    if (depth === 0) {
      keySet = new Set();
    } else {
      keys.forEach((key) => keySet.add(key));
    }
    // Get all own property names and symbols.
    keys = Reflect.ownKeys(obj);
    ctx.seen.push(main);
    for (const key of keys) {
      // Ignore the `constructor` property and keys that exist on layers above.
      if (
        key === "constructor" ||
        Object.hasOwn(main, key) ||
        // @ts-expect-error this is fine
        (depth !== 0 && keySet.has(key))
      ) {
        continue;
      }
      const desc = Object.getOwnPropertyDescriptor(obj, key)!;
      if (typeof desc.value === "function") {
        continue;
      }
      const value = formatProperty(
        ctx,
        // @ts-expect-error this is fine
        obj,
        recurseTimes,
        key,
        kObjectType,
        desc,
        main,
      );
      output.push(value);
    }
    ctx.seen.pop();
    // Limit the inspection to up to three prototype layers. Using `recurseTimes`
    // is not a good choice here, because it's as if the properties are declared
    // on the current object from the users perspective.
  } while (++depth !== 3);
}

function formatProperty(
  ctx: InspectOptions,
  value: { [key: PropertyKey]: unknown },
  recurseTimes: number,
  key: PropertyKey,
  type: number,
  desc?: PropertyDescriptor,
  original = value,
) {
  let name, str;
  let extra = " ";
  desc = desc ||
    Object.getOwnPropertyDescriptor(value, key) || {
      value: value[key],
      enumerable: true,
    };
  if (desc.value !== undefined) {
    const diff = ctx.compact !== true || type !== kObjectType ? 2 : 3;
    ctx.indentationLvl += diff;
    str = formatValue(ctx, desc.value, recurseTimes);
    if (diff === 3 && ctx.breakLength < getStringWidth(str, ctx.colors)) {
      extra = `\n${" ".repeat(ctx.indentationLvl)}`;
    }
    ctx.indentationLvl -= diff;
  } else if (desc.get !== undefined) {
    const label = desc.set !== undefined ? "Getter/Setter" : "Getter";
    const s = ctx.stylize;
    const sp = "special";
    if (
      ctx.getters &&
      (ctx.getters === true ||
        (ctx.getters === "get" && desc.set === undefined) ||
        (ctx.getters === "set" && desc.set !== undefined))
    ) {
      try {
        const tmp = desc.get.call(original);
        ctx.indentationLvl += 2;
        if (tmp === null) {
          str = `${s(`[${label}:`, sp)} ${s("null", "null")}${s("]", sp)}`;
        } else if (typeof tmp === "object") {
          str = `${s(`[${label}]`, sp)} ${formatValue(ctx, tmp, recurseTimes)}`;
        } else {
          const primitive = formatPrimitive(s, tmp, ctx);
          str = `${s(`[${label}:`, sp)} ${primitive}${s("]", sp)}`;
        }
        ctx.indentationLvl -= 2;
      } catch (err) {
        const message = `<Inspection threw (${(err as Error).message})>`;
        str = `${s(`[${label}:`, sp)} ${message}${s("]", sp)}`;
      }
    } else {
      str = ctx.stylize(`[${label}]`, sp);
    }
  } else if (desc.set !== undefined) {
    str = ctx.stylize("[Setter]", "special");
  } else {
    str = ctx.stylize("undefined", "undefined");
  }
  if (type === kArrayType) {
    return str;
  }
  if (typeof key === "symbol") {
    name = `[${ctx.stylize(maybeQuoteSymbol(key, ctx), "symbol")}]`;
  } else if (key === "__proto__") {
    name = "['__proto__']";
  } else if (desc.enumerable === false) {
    const tmp = (key as string).replace(strEscapeSequencesReplacer, escapeFn);

    name = `[${tmp}]`;
  } else if (keyStrRegExp.test(key as string)) {
    name = ctx.stylize(key as string, "name");
  } else {
    name = ctx.stylize(quoteString(key as string, ctx), "string");
  }
  return `${name}:${extra}${str}`;
}

function isInstanceof(proto: unknown, object: object) {
  try {
    return Object.prototype.isPrototypeOf.call(proto, object);
  } catch {
    return false;
  }
}

function getConstructorName(
  obj: object,
  ctx: InspectOptions,
  recurseTimes: number,
  protoProps: unknown,
): string | null {
  let firstProto;
  const tmp = obj;
  while (obj || isUndetectableObject(obj)) {
    let descriptor;
    try {
      descriptor = Object.getOwnPropertyDescriptor(obj, "constructor");
    } catch {
      /* this could fail */
    }
    if (
      descriptor !== undefined &&
      typeof descriptor.value === "function" &&
      (descriptor.value as VoidFunction).name !== "" &&
      isInstanceof((descriptor.value as VoidFunction).prototype, tmp)
    ) {
      if (
        protoProps !== undefined &&
        (firstProto !== obj || !builtInObjects.has(descriptor.value.name))
      ) {
        addPrototypeProperties(
          ctx,
          tmp,
          firstProto || tmp,
          recurseTimes,
          // @ts-expect-error this is fine
          protoProps,
        );
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
  const res: string | undefined = tmp.prototype.name;

  if (recurseTimes > ctx.depth && ctx.depth !== null) {
    return `${res} <Complex prototype>`;
  }

  const protoConstr: string | null = getConstructorName(
    // @ts-expect-error this is fine
    firstProto,
    ctx,
    recurseTimes + 1,
    protoProps,
  );

  if (protoConstr === null) {
    return `${res} <${inspect(firstProto, {
      ...ctx,
      depth: -1,
      __proto__: null,
    })}>`;
  }

  return `${res} <${protoConstr}>`;
}

// Print strings when they are inside of arrays or objects with quotes
function inspectValueWithQuotes(value: unknown, ctx: InspectOptions) {
  const abbreviateSize =
    typeof ctx.strAbbreviateSize === "undefined"
      ? STR_ABBREVIATE_SIZE
      : ctx.strAbbreviateSize;
  switch (typeof value) {
    case "string": {
      const trunc =
        value.length > abbreviateSize
          ? value.slice(0, abbreviateSize) + "..."
          : value;
      return ctx.stylize(quoteString(trunc, ctx), "string"); // Quoted strings are green
    }
    default:
      return formatValue(ctx, value, 0);
  }
}

const tableChars = {
  middleMiddle: "\u2500",
  rowMiddle: "\u253c",
  topRight: "\u2510",
  topLeft: "\u250c",
  leftMiddle: "\u251c",
  topMiddle: "\u252c",
  bottomRight: "\u2518",
  bottomLeft: "\u2514",
  bottomMiddle: "\u2534",
  rightMiddle: "\u2524",
  left: "\u2502 ",
  right: " \u2502",
  middle: " \u2502 ",
};

function hasOwnProperty(obj: object | null, v: PropertyKey) {
  if (obj == null) {
    return false;
  }
  return Object.hasOwn(obj, v);
}

function cliTable(head: string[], columns: string[][]) {
  const rows: string[][] = [];
  const columnWidths = head.map((h) => getStringWidth(h));
  const longestColumn = columns.reduce((n, a) => Math.max(n, a.length), 0);
  const columnRightAlign: boolean[] = new Array(columnWidths.length).fill(true);

  for (let i = 0; i < head.length; i++) {
    const column = columns[i];
    for (let j = 0; j < longestColumn; j++) {
      if (rows[j] === undefined) {
        rows[j] = [];
      }
      const value = (rows[j][i] = hasOwnProperty(column, j) ? column[j] : "");
      const width = columnWidths[i] || 0;
      const counted = getStringWidth(value);
      columnWidths[i] = Math.max(width, counted);
      columnRightAlign[i] = columnRightAlign[i] && Number.isInteger(+value);
    }
  }

  const divider = columnWidths.map((i) =>
    tableChars.middleMiddle.repeat(i + 2),
  );

  let result =
    `\n${tableChars.topLeft}${divider.join(tableChars.topMiddle)}` +
    `${tableChars.topRight}\n${renderRow(head, columnWidths)}\n` +
    `${tableChars.leftMiddle}${divider.join(tableChars.rowMiddle)}` +
    `${tableChars.rightMiddle}\n`;

  for (let i = 0; i < rows.length; ++i) {
    const row = rows[i];
    result += `${renderRow(row, columnWidths, columnRightAlign)}\n`;
  }

  result +=
    `${tableChars.bottomLeft}${divider.join(tableChars.bottomMiddle)}` +
    tableChars.bottomRight;

  return result;
}

function renderRow(
  row: string[],
  columnWidths: number[],
  columnRightAlign?: boolean[],
) {
  let out = tableChars.left;
  for (let i = 0; i < row.length; i++) {
    const cell = row[i];
    const len = getStringWidth(cell);
    const padding = " ".repeat(columnWidths[i] - len);
    if (columnRightAlign?.[i]) {
      out += `${padding}${cell}`;
    } else {
      out += `${cell}${padding}`;
    }
    if (i !== row.length - 1) {
      out += tableChars.middle;
    }
  }
  out += tableChars.right;
  return out;
}

function inspect(
  value: unknown,
  inspectOptions: Partial<InspectOptions> & { __proto__: null } = {
    __proto__: null,
  },
) {
  // Default options
  const ctx = {
    ...getDefaultInspectOptions(),
    ...inspectOptions,
  };

  return formatValue(ctx, value, 0);
}

const countMap = new Map<string, number>();
const timerMap = new Map<string, number>();
const isConsoleInstance = Symbol("isConsoleInstance");
type PrintFunc = (message: string, level: number) => void;

class Console {
  #printFunc: PrintFunc;
  indentLevel = 0;
  [isConsoleInstance] = false;

  constructor(printFunc: PrintFunc) {
    this.#printFunc = printFunc;
    this[isConsoleInstance] = true;
    this.indentLevel = 0;

    // ref https://console.spec.whatwg.org/#console-namespace
    // For historical web-compatibility reasons, the namespace object for
    // console must have as its [[Prototype]] an empty object, created as if
    // by ObjectCreate(%ObjectPrototype%), instead of %ObjectPrototype%.
    const console: Console = Object.create(
      {},
      {
        [Symbol.toStringTag]: {
          enumerable: false,
          writable: false,
          configurable: true,
          value: "console",
        },
      },
    );
    Object.assign(console, this);
    return console;
  }

  log = (...args: unknown[]) => {
    this.#printFunc(
      inspectArgs(args, {
        ...getDefaultInspectOptions(),
        indentLevel: this.indentLevel,
        __proto__: null,
      }) + "\n",
      1,
    );
  };

  debug = (...args: unknown[]) => {
    this.#printFunc(
      inspectArgs(args, {
        ...getDefaultInspectOptions(),
        indentLevel: this.indentLevel,
        __proto__: null,
      }) + "\n",
      0,
    );
  };

  info = (...args: unknown[]) => {
    this.#printFunc(
      inspectArgs(args, {
        ...getDefaultInspectOptions(),
        indentLevel: this.indentLevel,
        __proto__: null,
      }) + "\n",
      1,
    );
  };

  dir = (obj: unknown = undefined, options = { __proto__: null }) => {
    this.#printFunc(
      inspectArgs([obj], {
        ...getDefaultInspectOptions(),
        ...options,
      }) + "\n",
      1,
    );
  };

  dirxml = this.dir;

  warn = (...args: unknown[]) => {
    this.#printFunc(
      inspectArgs(args, {
        ...getDefaultInspectOptions(),
        indentLevel: this.indentLevel,
        __proto__: null,
      }) + "\n",
      2,
    );
  };

  error = (...args: unknown[]) => {
    this.#printFunc(
      inspectArgs(args, {
        ...getDefaultInspectOptions(),
        indentLevel: this.indentLevel,
        __proto__: null,
      }) + "\n",
      3,
    );
  };

  assert = (condition = false, ...args: unknown[]) => {
    if (condition) {
      return;
    }

    if (args.length === 0) {
      this.error("Assertion failed");
      return;
    }

    const [first, ...rest] = args;

    if (typeof first === "string") {
      this.error(`Assertion failed: ${first}`, ...rest);
      return;
    }

    this.error(`Assertion failed:`, ...args);
  };

  count = (label = "default") => {
    label = String(label);

    if (countMap.has(label)) {
      const current = countMap.get(label) || 0;
      countMap.set(label, current + 1);
    } else {
      countMap.set(label, 1);
    }

    this.info(`${label}: ${countMap.get(label)}`);
  };

  countReset = (label = "default") => {
    label = String(label);

    if (countMap.has(label)) {
      countMap.set(label, 0);
    } else {
      this.warn(`Count for '${label}' does not exist`);
    }
  };

  table = (data: unknown[] | undefined = undefined, properties: string[]) => {
    if (properties !== undefined && !Array.isArray(properties)) {
      throw new Error(
        "The 'properties' argument must be of type Array. " +
          "Received type " +
          typeof properties,
      );
    }

    if (data === null || typeof data !== "object") {
      return this.log(data);
    }

    const stringifyValue = (value: unknown): string =>
      inspectValueWithQuotes(value, {
        ...getDefaultInspectOptions(),
        depth: 1,
        compact: true,
      });
    const toTable = (header: string[], body: string[][]) =>
      this.log(cliTable(header, body));

    let resultData: unknown[];
    const isSetObject = isSet(data);
    const isMapObject = isMap(data);
    const valuesKey = "Values";
    const indexKey = isSetObject || isMapObject ? "(iter idx)" : "(idx)";

    if (isSetObject) {
      //
      resultData = [...data];
    } else if (isMapObject) {
      let idx = 0;
      // @ts-expect-error this is fine
      resultData = { __proto__: null };
      data.forEach((v, k) => {
        resultData[idx] = { Key: k, Values: v };
        idx++;
      });
    } else {
      resultData = data;
    }

    const keys = Object.keys(resultData);
    const numRows = keys.length;

    const objectValues: { [key: PropertyKey]: string[] } = properties
      ? Object.fromEntries(
          properties.map((name) => [name, new Array(numRows).fill("")]),
        )
      : {};
    const indexKeys: string[] = [];
    const values: string[] = [];

    let hasPrimitives = false;
    keys.forEach((k, idx) => {
      const value = (resultData as unknown as { [key: PropertyKey]: unknown })[
        k
      ];
      const primitive =
        value === null ||
        (typeof value !== "function" && typeof value !== "object");
      if (properties === undefined && primitive) {
        hasPrimitives = true;
        values.push(stringifyValue(value));
      } else {
        // @ts-expect-error this is fine
        const valueObj: { [key: PropertyKey]: unknown } = value || {};
        const keys = properties || Object.keys(valueObj);
        for (let i = 0; i < keys.length; ++i) {
          const k = keys[i];
          if (!primitive && Reflect.has(valueObj, k)) {
            if (!Reflect.has(objectValues, k)) {
              objectValues[k] = new Array(numRows).fill("");
            }
            objectValues[k][idx] = stringifyValue(valueObj[k]);
          }
        }
        values.push("");
      }

      indexKeys.push(k);
    });

    const headerKeys = Object.keys(objectValues);
    const bodyValues = Object.values(objectValues);
    const headerProps = properties || [
      ...headerKeys,
      !isMapObject && hasPrimitives && valuesKey,
    ];
    const header = [indexKey, ...headerProps].filter(Boolean);
    const body = [indexKeys, ...bodyValues, values];

    toTable(header, body);
  };

  time = (label = "default") => {
    label = String(label);

    if (timerMap.has(label)) {
      this.warn(`Timer '${label}' already exists`);
      return;
    }

    timerMap.set(label, Date.now());
  };

  timeLog = (label = "default", ...args: unknown[]) => {
    label = String(label);

    if (!timerMap.has(label)) {
      this.warn(`Timer '${label}' does not exist`);
      return;
    }

    const startTime = timerMap.get(label)!;
    const duration = Date.now() - startTime;

    this.info(`${label}: ${duration}ms`, ...args);
  };

  timeEnd = (label = "default") => {
    label = String(label);

    if (!timerMap.has(label)) {
      this.warn(`Timer '${label}' does not exist`);
      return;
    }

    const startTime = timerMap.get(label)!;
    timerMap.delete(label);
    const duration = Date.now() - startTime;

    this.info(`${label}: ${duration}ms`);
  };

  group = (...label: unknown[]) => {
    if (label.length > 0) {
      this.log(...label);
    }
    this.indentLevel += 2;
  };

  groupCollapsed = this.group;

  groupEnd = () => {
    if (this.indentLevel > 0) {
      this.indentLevel -= 2;
    }
  };

  clear = () => {
    this.indentLevel = 0;
    this.#printFunc("\x1b[1;1H", 1);
    this.#printFunc("\x1b[0J", 1);
  };

  trace = (...args: unknown[]) => {
    const message = inspectArgs(args, {
      ...getDefaultInspectOptions(),
      indentLevel: 0,
      __proto__: null,
    });
    const err: Error = {
      name: "Trace",
      message,
    };
    try {
      // @ts-expect-error this is fine
      Error.prototype.captureStackTrace.call(err, this.trace);
    } catch (err) {}
    this.error(err.stack);
  };
}

export { Console, inspect };
