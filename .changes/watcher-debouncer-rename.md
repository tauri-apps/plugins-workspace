---
"fs": patch
---

Fixes `RenameMode::From` and `RenameMode::To` never getting converted to `RenameMode::Both` when using `watch` with a debounce on Windows
