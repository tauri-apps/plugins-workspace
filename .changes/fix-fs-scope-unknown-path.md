---
fs: patch
---

Fixed an issue causing any `fs` APIs to throw an `error deserializing scope: unknown path` error when the permissions/scopes allowed OS specific paths that aren't available on the currently running OS.