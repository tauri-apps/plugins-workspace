---
single-instance: patch
---

fix `cwd` in single instance on macOS, which was the cwd of the first instance, instead of the second (like it is on windows and linux)
