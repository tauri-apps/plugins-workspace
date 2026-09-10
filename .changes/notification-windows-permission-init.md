---
"notification": patch
"notification-js": patch
---

On Windows, the init script no longer forces `window.Notification.permission` to `"denied"` at startup; it now asks the backend like every other platform.
