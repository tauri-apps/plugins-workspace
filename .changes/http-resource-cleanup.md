---
http: patch
http-js: patch
---

Fix HTTP resource leaks on cancellation, errors, navigation, and bodyless responses, including WebKit memory leaks. Clean up abandoned responses after garbage collection on supported webviews and reject response statuses outside 200–599.
