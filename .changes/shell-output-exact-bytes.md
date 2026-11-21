---
shell: patch
shell-js: patch
---

`Command::output` returns the bytes the process wrote. It appended a newline to every line, which already ends with its own line ending, so text output had every line break doubled, and raw output (`set_raw_out(true)`) had a newline inserted after every chunk, corrupting binary data.
