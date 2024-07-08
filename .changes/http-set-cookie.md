---
"http-js": "patch"
---

Fix missing `Set-Cookie` headers in the response which meant `request.headers.getSetCookie()` always returned empty array.
