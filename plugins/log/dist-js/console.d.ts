/// <reference lib="esnext" />
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
/**
 * Remove all VT control characters. Use to estimate displayed string width.
 */
export declare function stripVTControlCharacters(str: string): string;
export declare function getStringWidth(str: string, removeControlChars?: boolean): number;
declare function inspect(value: unknown, inspectOptions?: Partial<InspectOptions> & {
    __proto__: null;
}): string;
declare const isConsoleInstance: unique symbol;
type PrintFunc = (message: string, level: number) => void;
declare class Console {
    #private;
    indentLevel: number;
    [isConsoleInstance]: boolean;
    constructor(printFunc: PrintFunc);
    log: (...args: unknown[]) => void;
    debug: (...args: unknown[]) => void;
    info: (...args: unknown[]) => void;
    dir: (obj?: unknown, options?: {
        __proto__: null;
    }) => void;
    dirxml: (obj?: unknown, options?: {
        __proto__: null;
    }) => void;
    warn: (...args: unknown[]) => void;
    error: (...args: unknown[]) => void;
    assert: (condition?: boolean, ...args: unknown[]) => void;
    count: (label?: string) => void;
    countReset: (label?: string) => void;
    table: (data: unknown[] | undefined, properties: string[]) => void;
    time: (label?: string) => void;
    timeLog: (label?: string, ...args: unknown[]) => void;
    timeEnd: (label?: string) => void;
    group: (...label: unknown[]) => void;
    groupCollapsed: (...label: unknown[]) => void;
    groupEnd: () => void;
    clear: () => void;
    trace: (...args: unknown[]) => void;
}
export { Console, inspect };
