---
notification: patch
notification-js: patch
---

Fixed scheduled Android notifications being saved as `null` by persisting their current data, including schedule options and nested JSON. Unreadable saved records are skipped without deleting their IDs.
