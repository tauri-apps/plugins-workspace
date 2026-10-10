---
updater: patch
updater-js: patch
---

`check()`, `Update.download()` and `Update.downloadAndInstall()` no longer replace the `headers` of the options object they are given with an array.
