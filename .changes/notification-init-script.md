---
"notification": patch
---

Use `window.__TAURI_INVOKE__` instead of `window.__TAURI__` in init.js, fixes usage in apps without `withGlobalTauri` enabled.
