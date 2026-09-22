---
"fs": major
---

**Breaking:** Removed the `allow-unwatch` and `deny-unwatch` permissions. There has been no `unwatch` command since v2.0 (`Watcher.close()` releases the resource instead), so they never granted anything.
