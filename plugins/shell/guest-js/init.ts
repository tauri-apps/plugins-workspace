// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from "@tauri-apps/api/core";

// open <a href="..."> links with the API
function openLinks(): void {
  document.querySelector("body")?.addEventListener("click", function (e) {
    let target: HTMLElement | null = e.target as HTMLElement;
    while (target) {
      if (target.matches("a")) {
        const t = target as HTMLAnchorElement;
        if (
          t.href !== "" &&
          ["http://", "https://", "mailto:", "tel:"].some((v) =>
            t.href.startsWith(v),
          ) &&
          t.target === "_blank"
        ) {
          void invoke("plugin:shell|open", {
            path: t.href,
          });
          e.preventDefault();
        }
        break;
      }
      target = target.parentElement;
    }
  });
}

if (
  document.readyState === "complete" ||
  document.readyState === "interactive"
) {
  openLinks();
} else {
  window.addEventListener("DOMContentLoaded", openLinks, true);
}
