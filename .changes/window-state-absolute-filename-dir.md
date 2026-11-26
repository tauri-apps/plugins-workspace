---
window-state: patch
window-state-js: patch
---

Create the state file's own parent directory when saving, instead of the app config directory. With an absolute path from `Builder::with_filename`, the plugin no longer creates an empty app config directory on every save.
