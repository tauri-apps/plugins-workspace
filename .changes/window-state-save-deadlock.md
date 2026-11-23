---
window-state: patch
---

Avoid holding the `WindowStateCache` mutex across `Window::update_state` calls and file writes in `save_window_state` to prevent a deadlock when window move/resize events arrive on the main thread during a save.
