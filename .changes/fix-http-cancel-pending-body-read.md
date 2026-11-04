---
http: patch
http-js: patch
---

Fixed response body cancellation waiting for another chunk while a streaming server is silent. Aborting the request signal or canceling the body now interrupts pending reads and releases the response promptly.
