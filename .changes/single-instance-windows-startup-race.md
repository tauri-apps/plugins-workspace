---
"single-instance": patch
---

Fix race condition on Windows where a secondary instance fails to exit if `FindWindowW` misses the target event window during startup.
