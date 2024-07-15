---
"updater": patch
---

On Windows, use a named tempfile with `<app name>-<version>-installer.exe` (or `.msi`) for v2 updater

**Breaking Change**: `UpdaterBuilder::new` now takes one more argument `app_name: String`
