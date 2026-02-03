---
"single-instance": minor:fix
---

**Breaking Change:** On Linux, the DBus ID/name will now be `<bundle-id>.SingleInstance` instead of `org.<bundle_id_underscores>.SingleInstance` to follow DBus specifications.

This will break the single-instance mechanism across different app versions if the app was installed multiple times.

Added `dbus_id` builder method, which can be used to restore previous behavior. For a bundle identifier of `com.tauri.my-example` this would be `dbus_id("org.com_tauri_my_example")`.