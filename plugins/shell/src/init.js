// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

(function () {
  // open <a href="..."> links with the API
  function openLinks() {
    document.querySelector("body").addEventListener("click", function (e) {
      var target = e.target;
      while (target != null) {
        if (target.matches("a")) {
          if (
            target.href &&
            ["http://", "https://", "mailto:", "tel:"].some((v) =>
              target.href.startsWith(v),
            ) &&
            target.target === "_blank"
          ) {
            invoke("plugin:shell|open", {
              path: target.href,
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
})();
