---
"fs-js": major
---

**Breaking:** `watch` and `watchImmediate` now resolve to a `Watcher` resource instead of an `UnwatchFn` callback. Call `await watcher.close()` to stop watching. The `UnwatchFn` type has been removed and `Watcher` is now exported.
