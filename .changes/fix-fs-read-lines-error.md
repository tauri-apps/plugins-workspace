---
fs: patch
fs-js: patch
---

Fixed `readTextFileLines` yielding empty lines forever when reading fails (e.g. on a directory). The read error is now reported: the iterator rejects and the file is closed.
