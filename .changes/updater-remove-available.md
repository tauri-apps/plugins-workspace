---
"updater-js": major
"updater": major
---

**Breaking:** Removed the deprecated `Update.available` field, which was always `true`. Check whether `check()` returned `null` instead.
