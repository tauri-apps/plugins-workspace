---
"window-state": patch
---

Modify the condition for calling `window.is_maximized()` in the `update_state` method, ensuring it's only called when the `MAXIMIZED` flag is set in `StateFlags`.
