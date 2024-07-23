---
"log": patch
---

Added `Builder::split` which returns the raw logger implementation so you can pipe to other loggers such as `multi_log` or `tauri-plugin-devtools`.
