---
notification: patch
notification-js: patch
---

Skip automatic notification permission queries on startup `about:blank` documents and handle failed startup queries without an unhandled promise rejection. Explicit permission queries and requests can still retry.
