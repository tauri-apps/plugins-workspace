---
http: minor
http-js: minor
---

Added the plugin `Builder`, which sets the `userAgent` and `requestHeaders` options in Rust, plus an `on_request` hook that can change the headers of every request. `init()` is now equivalent to `Builder::new().build()`.
