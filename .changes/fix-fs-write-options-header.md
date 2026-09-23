---
fs: patch
fs-js: patch
---

`writeFile` and `writeTextFile` now fail when their options cannot be parsed instead of silently ignoring them (including `baseDir`, `append` and `createNew`) and writing to the raw path. Data sent as a JSON array is rejected if it contains values that are not bytes, instead of truncating them.
