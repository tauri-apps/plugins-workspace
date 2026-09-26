---
"notification": minor
---

Deprecate the `windows7-compat` Cargo feature, which is now a no-op since Tauri 2.12 no longer supports Windows 7. `NotificationBuilder::notify` is deprecated in favor of `NotificationBuilder::show`, and the `win7-notifications` and `windows-version` dependencies were removed.
