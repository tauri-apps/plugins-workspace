---
"window-state-js": "patch"
---

Correctly propagate the promise inside `saveWindowState`, `restoreState` and `restoreStateCurrent` so callers can choose to `await` them.
