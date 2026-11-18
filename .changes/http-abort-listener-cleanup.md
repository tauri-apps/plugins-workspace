---
http: patch
http-js: patch
---

Fix request cancellation, including signals supplied through `Request` objects, abort-listener cleanup, and concurrent response-body reads.
