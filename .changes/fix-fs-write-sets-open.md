---
fs: patch
fs-js: patch
---

The `write-all` and `write-files` permissions (and so every `allow-*-write[-recursive]` set) are now sets of the `allow-*` command permissions, so they also allow the commands a command depends on. `writeFile` with a `ReadableStream` uses `open` and failed with "not allowed" under these sets.
