---
updater: patch
updater-js: patch
---

Fixed an issue that caused the plugin to emit a `ReleaseNotFound` error instead of a `Reqwest` error when the http request in `check()` failed.
