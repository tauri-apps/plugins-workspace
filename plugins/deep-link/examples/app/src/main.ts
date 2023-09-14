// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import {
  onOpenUrl,
  getCurrent as getCurrentDeepLinkUrls,
} from "@tauri-apps/plugin-deep-link";

function handler(urls: string[]) {
  console.log(urls);
  // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
  const updateIntentEl = document.querySelector("#event-intent")!;
  updateIntentEl.textContent = JSON.stringify(urls);
}

window.addEventListener("DOMContentLoaded", () => {
  onOpenUrl(handler);

  document.querySelector("#intent-form")?.addEventListener("submit", (e) => {
    e.preventDefault();
    getCurrentDeepLinkUrls()
      .then((res) => {
        // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
        const updateIntentEl = document.querySelector("#update-intent")!;
        updateIntentEl.textContent = res ? JSON.stringify(res) : "none";
      })
      .catch(console.error);
  });

  getCurrentDeepLinkUrls()
    .then((res) => {
      // eslint-disable-next-line @typescript-eslint/no-non-null-assertion
      const initialIntentEl = document.querySelector("#initial-intent")!;
      initialIntentEl.textContent = res ? JSON.stringify(res) : "none";
    })
    .catch(console.error);
});
