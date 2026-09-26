---
fs: patch
fs-js: patch
---

Fixed a panic on Android when a content provider returns no file descriptor for a `content://` URI (e.g. virtual files). The fs command now fails with an error instead.
