---
fs: patch
fs-js: patch
---

Fixed `writeFile` with a `ReadableStream` leaving the end of the previous contents in place when the new data is shorter. The file is now truncated first unless `append` is set, like when writing a buffer.
