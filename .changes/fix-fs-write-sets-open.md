---
fs: patch
fs-js: patch
---

The `write-all` and `write-files` permissions (and so every `allow-*-write[-recursive]` set) now also allow the `open`, `seek` and `fstat` commands. `writeFile` with a `ReadableStream` uses `open` and failed with "not allowed" under these sets, and the `FileHandle` returned by `create` could not `seek` or `stat`.
