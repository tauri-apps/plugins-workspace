---
notification: patch
notification-js: patch
---

Fixed scheduled notifications on Android being saved as `null`, which broke `pending()` and lost them across a device reboot. `actionPerformed` events now include the notification on every path, the pending schedule reports `allowWhileIdle`, and scheduled notifications fire the `notification` event (`onNotificationReceived`) while the app is running.
