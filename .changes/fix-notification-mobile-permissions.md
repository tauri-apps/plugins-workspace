---
notification: patch
notification-js: patch
---

Fixed broken mobile APIs:

- `channels()` invoked the `listChannels` command, which the `allow-list-channels` permission does not match; it now invokes `list_channels`.
- Unregistering an `onNotificationReceived` or `onAction` listener invokes the `remove_listener` command, which had no permission. It is now declared and part of the `default` permission set (`allow-remove-listener`).
- `cancelAll()` was rejected on Android and iOS, which required the list of notifications to cancel. It now cancels every pending notification.
- `active()` resolved with `{ values: [...] }` instead of an array on Android.
