---
"notification": patch
"notification-js": patch
---

On Linux and the BSDs, fixed `removeActive` and `removeAllActive` (and `Notification::remove_active` and `remove_all_active` called from an async task) panicking with "Cannot start a runtime from within a runtime" when another crate in the app enables zbus's `tokio` feature (ashpd does by default). The notifications are now closed.
