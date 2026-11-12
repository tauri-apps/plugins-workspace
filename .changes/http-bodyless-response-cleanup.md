---
http: patch
http-js: patch
---

Release native HTTP responses for HEAD requests and null-body statuses before returning them to JavaScript. Reject response statuses outside 200–599 without retaining native resources.
