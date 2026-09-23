---
fs: patch
fs-js: patch
---

Fixed opening Android `content://` URIs with `append` or with `write`, `truncate` and `append` together: the options were turned into invalid modes such as `ra` or `wta`. They now map to the modes Android accepts (`r`, `w`, `wt`, `wa`, `rw`, `rwt`), `append` taking precedence over `read` since there is no read and append mode.
