---
"window-state": minor:feat
"window-state-js": minor:feat
---

Add `StateFlags::MINIMIZED` to save and restore whether the window is minimized. It is not part of `StateFlags::default()`, so it must be enabled explicitly. Size and position are no longer saved while the window is fullscreen.
