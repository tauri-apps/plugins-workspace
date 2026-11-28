---
shell: minor
shell-js: minor
---

`Command::output` now returns exactly the bytes the process wrote. It used to append a newline after every line and every raw chunk, so update code that compares the output or strips the extra newlines.

