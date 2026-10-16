---
http: minor
http-js: minor
---

Added the `HttpExt` trait. `app.http().cookie_jar()` returns the plugin's `CookieJar`, which can be passed to your own `reqwest` clients to share the webview's session, and can be cleared or saved.
