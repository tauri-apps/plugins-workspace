---
log: patch
log-js: patch
---

Fixed `RotationStrategy::KeepSome` deleting the newest rotated log files when same-second rotation names reach the `_10` collision suffix. Retention now compares file modification times instead of sorting rotated file names lexicographically, so a `<name>_<date>.log` name reused after cleanup is also kept correctly.
