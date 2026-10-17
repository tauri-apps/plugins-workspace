---
deep-link: patch
deep-link-js: patch
---

On Linux, `register` and `unregister` no longer fail when `update-desktop-database` is not installed. A warning is logged and the MIME cache update is skipped.
