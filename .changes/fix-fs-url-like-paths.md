---
fs: patch
fs-js: patch
---

Fixed relative paths whose first component contains a colon, such as `notes:2024.txt`, being parsed as URLs and rejected with "URL is not a valid path". Only strings with a hierarchical URL form (`scheme://...` or `scheme:/...`) are treated as URLs now.
