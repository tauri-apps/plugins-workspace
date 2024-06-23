/** user-defined commands **/
export declare const commands: {
    vibrate(duration: number): Promise<Result<null, Error>>;
    impactFeedback(style: ImpactFeedbackStyle): Promise<Result<null, Error>>;
    notificationFeedback(type: NotificationFeedbackType): Promise<Result<null, Error>>;
    selectionFeedback(): Promise<Result<null, Error>>;
};
/** user-defined events **/
export declare const events: {
    randomNumber: __EventObj__<number> & ((handle: __WebviewWindow__) => __EventObj__<number>);
};
/** user-defined statics **/
/** user-defined types **/
export type Error = never;
export type ImpactFeedbackStyle = "light" | "medium" | "heavy" | "soft" | "rigid";
export type NotificationFeedbackType = "success" | "warning" | "error";
export type RandomNumber = number;
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";
type __EventObj__<T> = {
    listen: (cb: TAURI_API_EVENT.EventCallback<T>) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
    once: (cb: TAURI_API_EVENT.EventCallback<T>) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
    emit: T extends null ? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit> : (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};
export type Result<T, E> = {
    status: "ok";
    data: T;
} | {
    status: "error";
    error: E;
};
export {};
