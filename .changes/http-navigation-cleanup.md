---
http: patch
http-js: patch
---

Cancel pending HTTP requests and release response bodies when the webview starts loading a new page, including reloads. Requests still being prepared for the previous page are rejected instead of retained.
