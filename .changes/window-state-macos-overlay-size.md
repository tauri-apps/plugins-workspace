---
window-state: patch
window-state-js: patch
---

Fixed windows using `titleBarStyle: Overlay` on macOS growing by the title bar height every time their size was restored. Also fixed a maximized window being restored off-screen when the position it had before being maximized is on a monitor that is no longer connected.
