---
"http": patch
"http-js": patch
---

Fix `fetch` blocking until the whole response is read even if it was a streaming response.
