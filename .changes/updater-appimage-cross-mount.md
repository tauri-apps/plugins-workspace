---
updater: patch
---

On Linux, installing an AppImage update no longer fails with `Invalid cross-device link (os error 18)` when the temp or cache directory is a separate mount of the AppImage's file system, such as a bind-mounted or private `/tmp`. The backup now moves on to the next location, ending with the AppImage's own directory.
