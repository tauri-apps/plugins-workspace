---
"notification": patch
"notification-js": patch
---

Fix `cancel` command crashing on Android with `UninitializedPropertyAccessException` when invoked with no `notifications` argument — i.e. when the JS `cancelAll()` API or the Rust `NotificationExt::cancel_all()` SDK method is used. `CancelArgs.notifications` is now an optional list defaulting to `[]`; the handler routes an empty list to a new `TauriNotificationManager.cancelAll()` that enumerates saved notification IDs from storage and cancels each.
