---
fs: patch
fs-js: patch
---

Fixed `readTextFileLines` with a UTF-16 encoding splitting lines on a `0x0A 0x00` (LE) or `0x00 0x0A` (BE) byte sequence that is not aligned on a code unit, e.g. `ਗĀ`, which garbled every following line.
