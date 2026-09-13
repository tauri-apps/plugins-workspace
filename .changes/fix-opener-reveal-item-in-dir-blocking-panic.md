---
"opener": "patch"
"opener-js": "patch"
---

Fixed `revealItemInDir` panicking on Linux (and other zbus-based platforms) with `Cannot start a runtime from within a runtime` when called from an async command context. The blocking D-Bus call now runs via `spawn_blocking` instead of on the async runtime worker thread.
