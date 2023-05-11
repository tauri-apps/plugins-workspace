// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * The event system allows you to emit events to the backend and listen to events from it.
 *
 * @module
 */

import { invoke, transformCallback } from "@tauri-apps/api/tauri";

type EventName = string;

interface Event<T> {
  /** Event name */
  event: EventName;
  /** The label of the window that emitted this event. */
  windowLabel: string;
  /** Event identifier used to unlisten */
  id: number;
  /** Event payload */
  payload: T;
}

type EventCallback<T> = (event: Event<T>) => void;

type UnlistenFn = () => void;

/**
 * Listen to an event from the backend.
 *
 * @example
 * ```typescript
 * import { listen } from 'tauri-plugin-event-api';
 * const unlisten = await listen<string>('error', (event) => {
 *   console.log(`Got error in window ${event.windowLabel}, payload: ${event.payload}`);
 * });
 *
 * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
 * unlisten();
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @param handler Event handler callback.
 * @returns A promise resolving to a function to unlisten to the event.
 * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
 *
 * @since 1.0.0
 */
async function listen<T>(
  event: EventName,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  return await invoke<number>("plugin:event|listen", {
    event,
    handler: transformCallback(handler),
  }).then((eventId) => {
    return async () => _unlisten(event, eventId);
  });
}

async function _unlisten(event: string, eventId: number): Promise<void> {
  return await invoke("plugin:event|unlisten", {
    event,
    eventId,
  });
}

/**
 * Listen to an one-off event from the backend.
 *
 * @example
 * ```typescript
 * import { once } from 'tauri-plugin-event-api';
 * interface LoadedPayload {
 *   loggedIn: boolean,
 *   token: string
 * }
 * const unlisten = await once<LoadedPayload>('loaded', (event) => {
 *   console.log(`App is loaded, loggedIn: ${event.payload.loggedIn}, token: ${event.payload.token}`);
 * });
 *
 * // you need to call unlisten if your handler goes out of scope e.g. the component is unmounted
 * unlisten();
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 * @returns A promise resolving to a function to unlisten to the event.
 * Note that removing the listener is required if your listener goes out of scope e.g. the component is unmounted.
 *
 * @since 1.0.0
 */
async function once<T>(
  event: EventName,
  handler: EventCallback<T>
): Promise<UnlistenFn> {
  return listen<T>(event, (eventData) => {
    handler(eventData);
    _unlisten(event, eventData.id);
  });
}

/**
 * Emits an event to the backend.
 * @example
 * ```typescript
 * import { emit } from 'tauri-plugin-event-api';
 * await emit('frontend-loaded', { loggedIn: true, token: 'authToken' });
 * ```
 *
 * @param event Event name. Must include only alphanumeric characters, `-`, `/`, `:` and `_`.
 *
 * @since 1.0.0
 */
async function emit(event: string, payload?: unknown): Promise<void> {
  return await invoke("plugin:event|emit", {
    event,
    payload,
  });
}

export type { Event, EventName, EventCallback, UnlistenFn };

export { listen, once, emit };
