---
"single-instance": patch
---

A second instance no longer panics when a command-line argument is not valid Unicode: such an argument is forwarded with its invalid parts replaced by U+FFFD. A working directory that is not valid Unicode is now forwarded the same way instead of as an empty string.
