---
http: patch
http-js: patch
---

Release native HTTP response resources for responses with no body, such as `204 No Content`, even when the response is never read.
