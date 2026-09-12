---
"http": patch
"http-js": patch
---

Fix unhandled promise rejections on every `fetch` teardown: the request/body cleanup commands were fired as floating promises, and releasing an already-released resource rejects with `The resource id N is invalid.`. `dropBody` is now idempotent and both cleanup calls handle their own rejection.
