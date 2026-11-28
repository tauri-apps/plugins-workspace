---
"updater": patch
"updater-js": patch
---

Fix compilation on BSD by keeping AppImage handling Linux-only. BSD applications can use a custom target to check release metadata; installation returns an unsupported OS error.
