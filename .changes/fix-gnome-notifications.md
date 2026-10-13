---
notification: patch
notification-js: patch
---

Fix notifications closing right after they show on GNOME: on Linux and the BSDs, every notification is now sent on the D-Bus connection the plugin keeps open.
