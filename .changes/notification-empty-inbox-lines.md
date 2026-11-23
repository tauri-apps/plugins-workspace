---
notification: patch
---

Skip serializing empty `inbox_lines` on `NotificationData` and guard against empty `inboxLines` lists on Android so notifications with only a title and body keep their body visible when expanded.
