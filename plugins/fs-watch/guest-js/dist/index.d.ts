import { UnlistenFn } from "@tauri-apps/api/event";
export interface WatchOptions {
    recursive?: boolean;
}
export interface DebouncedWatchOptions extends WatchOptions {
    delayMs?: number;
}
export interface RawEvent {
    path: string | null;
    operation: number;
    cookie: number | null;
}
export type DebouncedEvent = {
    type: "NoticeWrite";
    payload: string;
} | {
    type: "NoticeRemove";
    payload: string;
} | {
    type: "Create";
    payload: string;
} | {
    type: "Write";
    payload: string;
} | {
    type: "Chmod";
    payload: string;
} | {
    type: "Remove";
    payload: string;
} | {
    type: "Rename";
    payload: string;
} | {
    type: "Rescan";
    payload: null;
} | {
    type: "Error";
    payload: {
        error: string;
        path: string | null;
    };
};
export declare function watch(paths: string | string[], options: DebouncedWatchOptions, cb: (event: DebouncedEvent) => void): Promise<UnlistenFn>;
export declare function watchImmediate(paths: string | string[], options: WatchOptions, cb: (event: RawEvent) => void): Promise<UnlistenFn>;
