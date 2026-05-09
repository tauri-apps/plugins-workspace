---
"tauri-plugin-window-state": patch
"@tauri-apps/plugin-window-state": patch
---

Refactored state restoration and window caching to use `Logical` coordinates instead of `Physical`, resolving scaling drift and position anomalies across multi-monitor setups on macOS. Implemented a specific heuristic to compensate for `titleBarStyle: Overlay` causing cascading window shrinkage.
