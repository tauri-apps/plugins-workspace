---
"fs": major
"fs-js": major
---

**Breaking:** Removed the `read_text_file` and `write_text_file` commands and their `allow-read-text-file`, `deny-read-text-file`, `allow-write-text-file` and `deny-write-text-file` permissions. `readTextFile` and `writeTextFile` now use the `read_file` and `write_file` commands, so grant `fs:allow-read-file` and `fs:allow-write-file` instead.
