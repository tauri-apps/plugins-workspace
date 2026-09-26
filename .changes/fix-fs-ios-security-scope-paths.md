---
fs: patch
fs-js: patch
---

`startAccessingSecurityScopedResource` and `stopAccessingSecurityScopedResource` now accept absolute paths on iOS, as documented, instead of silently doing nothing unless given a `file://` URL.
