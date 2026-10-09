---
fs: patch
fs-js: patch
---

`FileHandle.seek` now rejects a negative offset with `SeekMode.Start` instead of seeking to a huge position.
