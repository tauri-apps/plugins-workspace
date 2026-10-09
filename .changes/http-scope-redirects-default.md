---
"http": major
"http-js": major
---

**Breaking:** The URL scope is now always checked on every hop of a redirect chain, so every redirect target must be allowed by the scope or the request fails with `url not allowed on the configured scope`. This was previously opt-in through the `scopeRedirects` plugin configuration, which has been removed along with the `Config` struct: `tauri_plugin_http::init()` returns `TauriPlugin<R>` again, and any `plugins > http` object must be removed from `tauri.conf.json`.
