---
"dialog": patch
"dialog-js": patch
---

The `open` function now returns a string representing either the file path or URI instead of an object.
To read the file data, use the `fs` APIs.
