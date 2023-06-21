---
"notification": patch
---

Revert [7d71ad4e5](https://github.com/tauri-apps/plugins-workspace/commit/7d71ad4e587bcf47ea34645f5b226945e487b765) which added a default sound for notifications on Windows. This introduced inconsistency with other platforms that has silent notifications by default. In the upcoming releases, we will add support for modifying the notification sound across all platforms.
