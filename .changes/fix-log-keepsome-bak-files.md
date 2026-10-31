---
"log": patch
"log-js": patch
---

Fixed `RotationStrategy::KeepSome` never deleting `.log.bak` files created by same-second rotations, so the log directory could grow past the configured cap. Same-second rotations now pick a unique `<name>_<date>_<n>.log` name instead of overwriting the existing `.log.bak`, and the cleanup counts legacy `.log.bak` files against the keep cap.
