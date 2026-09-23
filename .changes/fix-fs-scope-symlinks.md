---
fs: patch
fs-js: patch
---

**Security:** the fs scope check now resolves symlinks in the ancestors of paths that do not exist yet, and resolves relative symlink targets against the link's directory instead of the process working directory. Previously a new file created below a symlinked directory inside the scope (`writeFile`, `mkdir`, `create`, `rename`, `copyFile`, ...) was written outside of the scope. Deny patterns now also apply to the path as given, so they can name a symlink, and a path whose symlinks cannot be resolved (e.g. a loop) is rejected.
