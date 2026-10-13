---
clipboard-manager: minor
clipboard-manager-js: minor
---

Added `writeSecret` (`Clipboard::write_secret` in Rust) to write sensitive text, such as passwords, to the clipboard. Clipboard managers are asked to keep it out of their history on Windows, macOS and Linux. On Android the clip is flagged as sensitive, and on iOS it is not synced to other devices.
