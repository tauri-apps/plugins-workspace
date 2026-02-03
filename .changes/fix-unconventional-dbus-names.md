---
"single-instance": minor:fix
---

On Linux, the DBus ID/name will now be `<bundle-id>.SingleInstance` instead of `org.<bundle_id_underscores>.SingleInstance` to follow DBus specifications to primarily fix FlatPak compatibility.
**Breaking:** This will break the single-instance mechanism across multiple app versions which should only matter for very few use-cases where multiple installs of the same app are to be expected.

Added `dbus_id` builder method to set the ID manually, which can be used to restore previous behavior. For an example bundle identifier of `com.tauri.my-example` this would be `dbus_id("org.com_tauri_my_example.SingleInstance")`.