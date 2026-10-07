---
"http": patch
"http-js": patch
---

`fetch_send` now releases the request's resources once the send settles. Before, a request that got a response or failed to send left its `FetchRequest` and abort sender in the webview's resource table until the webview was destroyed, so a long-lived app grew with every request.
