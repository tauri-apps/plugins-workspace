---
http-js: patch
---

Added the cancel hook to ReadableStream to ensure resources are released via dropBody() when a consumer stops reading.
