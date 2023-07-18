// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { listen } from "@tauri-apps/api/event";
import { transformCallback } from "@tauri-apps/api/tauri";

declare global {
  interface Window {
    __TAURI_INVOKE__: <T>(cmd: string, args?: unknown) => Promise<T>;
  }
}

function handler(event: { payload?: string }) {
  console.log(event);
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const updateIntentEl = document.querySelector("#event-intent")!;
  updateIntentEl.textContent = event.payload ?? "empty event";
}

window.addEventListener("DOMContentLoaded", () => {
  listen("deep-link://new-url", console.log);
  // TODO: Replace with `listen` on next alpha
  window.__TAURI_INVOKE__<number>("plugin:event|listen", {
    event: "deep-link://new-url",
    windowLabel: "main",
    handler: transformCallback(handler),
  });

  document.querySelector("#intent-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    window
      .__TAURI_INVOKE__<string | null>("plugin:deep-link|get_last_link")
      .then((res) => {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        const updateIntentEl = document.querySelector("#update-intent")!;
        updateIntentEl.textContent = res ?? "none";
      })
      .catch(console.error);
  });

  window
    .__TAURI_INVOKE__<string | null>("plugin:deep-link|get_last_link")
    .then((res) => {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const initialIntentEl = document.querySelector("#initial-intent")!;
      initialIntentEl.textContent = res ?? "none";
    })
    .catch(console.error);
});
