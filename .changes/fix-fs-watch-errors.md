---
fs: patch
fs-js: patch
---

File watcher errors (e.g. the watched directory was removed or the inotify limit was reached) are now logged instead of being silently dropped.
