---
updater: patch
updater-js: patch
---

On Android and iOS, `Update::install` now returns `Error::UnsupportedOs` instead of reporting success without installing anything.
