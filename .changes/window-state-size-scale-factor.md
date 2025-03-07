---
window-state: patch
---

fix: The window size state needs to be related to the screen's scale_factor, Otherwise, the restored window size will be scaled relative to the previous state.
