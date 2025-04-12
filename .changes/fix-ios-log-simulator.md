---
"log": patch
"log-js": patch
---

Fix iOS app stuck when using the iOS Simulator and the log plugin due to a deadlock when calling os_log too early.
