---
fs: patch
fs-js: patch
---

`FileHandle.read` no longer allocates a buffer of the requested size up front: the length is capped to the rest of the file (64 MiB for files of unknown size), an allocation failure is reported as an error instead of aborting the app, and only the bytes that were read are sent back to the webview.
