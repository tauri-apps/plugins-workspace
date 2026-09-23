---
fs: patch
fs-js: patch
---

Fixed `readTextFileLines` leaving the file open until the webview is destroyed when a `for await` loop over it exits early (`break`, `return` or `throw`). The iterator now implements `return()`, which closes the file.
