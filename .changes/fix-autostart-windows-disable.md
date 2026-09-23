---
autostart: patch
---

Fixed `disable()` failing on Windows when autostart is already disabled. It now succeeds, as it does on macOS and Linux.
