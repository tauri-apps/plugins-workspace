---
"fs-js": "patch"
---

Fix `writeTextFile` converting UTF-8 characters (for example `äöü`) in the given path into replacement character (`�`)