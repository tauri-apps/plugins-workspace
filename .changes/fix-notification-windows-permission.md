---
notification: patch
notification-js: patch
---

Fixed `isPermissionGranted()` always resolving to `false` on Windows. The initialization script short-circuited the permission check to avoid invoking the backend, but read the permission back before it had been set, so it settled on `denied` instead of `granted`.
