// @ts-nocheck
// This file was generated by [tauri-specta](https://github.com/oscartbeaumont/tauri-specta). Do not edit this file manually.

/** user-defined commands **/

export const commands = {
  async vibrate(duration: number): Promise<Result<null, Error>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE("plugin:haptics|vibrate", { duration }),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async impactFeedback(
    style: ImpactFeedbackStyle
  ): Promise<Result<null, Error>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE("plugin:haptics|impact_feedback", { style }),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async notificationFeedback(
    type: NotificationFeedbackType
  ): Promise<Result<null, Error>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE("plugin:haptics|notification_feedback", {
          type,
        }),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
  async selectionFeedback(): Promise<Result<null, Error>> {
    try {
      return {
        status: "ok",
        data: await TAURI_INVOKE("plugin:haptics|selection_feedback"),
      };
    } catch (e) {
      if (e instanceof Error) throw e;
      else return { status: "error", error: e as any };
    }
  },
};

/** user-defined events **/

/* export const events = __makeEvents__<{
  randomNumber: RandomNumber;
}>({
  randomNumber: "plugin:haptics:random-number",
}); */

/** user-defined statics **/

/** user-defined types **/

export type Error = never;
export type ImpactFeedbackStyle =
  | "light"
  | "medium"
  | "heavy"
  | "soft"
  | "rigid";
export type NotificationFeedbackType = "success" | "warning" | "error";
//export type RandomNumber = number;

/** tauri-specta globals **/

import { invoke as TAURI_INVOKE } from "@tauri-apps/api/core";
import * as TAURI_API_EVENT from "@tauri-apps/api/event";
import { type WebviewWindow as __WebviewWindow__ } from "@tauri-apps/api/webviewWindow";

type __EventObj__<T> = {
  listen: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.listen<T>>;
  once: (
    cb: TAURI_API_EVENT.EventCallback<T>
  ) => ReturnType<typeof TAURI_API_EVENT.once<T>>;
  emit: T extends null
    ? (payload?: T) => ReturnType<typeof TAURI_API_EVENT.emit>
    : (payload: T) => ReturnType<typeof TAURI_API_EVENT.emit>;
};

export type Result<T, E> =
  | { status: "ok"; data: T }
  | { status: "error"; error: E };

function __makeEvents__<T extends Record<string, any>>(
  mappings: Record<keyof T, string>
) {
  return new Proxy(
    {} as unknown as {
      [K in keyof T]: __EventObj__<T[K]> & {
        (handle: __WebviewWindow__): __EventObj__<T[K]>;
      };
    },
    {
      get: (_, event) => {
        const name = mappings[event as keyof T];

        return new Proxy((() => {}) as any, {
          apply: (_, __, [window]: [__WebviewWindow__]) => ({
            listen: (arg: any) => window.listen(name, arg),
            once: (arg: any) => window.once(name, arg),
            emit: (arg: any) => window.emit(name, arg),
          }),
          get: (_, command: keyof __EventObj__<any>) => {
            switch (command) {
              case "listen":
                return (arg: any) => TAURI_API_EVENT.listen(name, arg);
              case "once":
                return (arg: any) => TAURI_API_EVENT.once(name, arg);
              case "emit":
                return (arg: any) => TAURI_API_EVENT.emit(name, arg);
            }
          },
        });
      },
    }
  );
}
