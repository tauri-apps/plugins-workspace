---
"single-instance": patch
---

On Windows, a second instance no longer panics when a command-line argument is not valid Unicode: such an argument is forwarded with its invalid parts replaced by U+FFFD.
