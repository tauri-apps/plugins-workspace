---
updater: patch
updater-js: patch
---

Updated HTTP cache dependencies to fix stale response handling during revalidation and send `Cache-Control: no-cache` on cache misses when using `checkNow`. These fixes apply when the `cache` feature is enabled.
