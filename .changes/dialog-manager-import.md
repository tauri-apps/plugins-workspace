---
"dialog": patch
"dialog-js": patch
---

Fixed the desktop build against Tauri 3.0.0-alpha.3, where `run_on_main_thread` moved to the `Manager` trait, by importing `Manager`.
