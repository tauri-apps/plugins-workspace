/** user-defined commands **/
export declare const commands: {
    vibrate(duration: number): Promise<Result<null, Error>>;
    impactFeedback(style: ImpactFeedbackStyle): Promise<Result<null, Error>>;
    notificationFeedback(type: NotificationFeedbackType): Promise<Result<null, Error>>;
    selectionFeedback(): Promise<Result<null, Error>>;
};
/** user-defined events **/
/** user-defined statics **/
/** user-defined types **/
export type Error = never;
export type ImpactFeedbackStyle = "light" | "medium" | "heavy" | "soft" | "rigid";
export type NotificationFeedbackType = "success" | "warning" | "error";
export type Result<T, E> = {
    status: "ok";
    data: T;
} | {
    status: "error";
    error: E;
};
