---
window-state: patch
window-state-js: patch
---

Fixed the `saveWindowState` JavaScript command deadlocking when window resize events occur during a save by running the save on the main thread.
