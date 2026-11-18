---
http: patch
http-js: patch
---

Fix request cancellation, including signals supplied through `Request` objects, abort-listener cleanup, and concurrent response-body reads. `fetch_cancel` and `fetch_cancel_body` now only close the plugin's own requests and responses, instead of any resource id they are given.
