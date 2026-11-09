---
notification: patch
---

On Linux, when running inside a Flatpak, notifications are now sent through the XDG Desktop Portal's Notification D-Bus. Limitations when using the portal: `on_close` callbacks only run if the close is triggered by the application, sound values only support 'default' or 'silent', and `file://` URIs for the notification icon don't work.
