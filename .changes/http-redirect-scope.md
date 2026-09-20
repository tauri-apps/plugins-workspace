---
"http": minor
---

**Security:** Added the `scopeRedirects` plugin configuration option, which checks the URL scope on every hop of a redirect chain instead of only on the URL requested by the frontend. Without it, a server on an allowed origin can redirect the request to any other origin - including `localhost` services, internal hosts and cloud metadata endpoints - and the plugin follows it, returning the response to the webview.

```json
{
  "plugins": {
    "http": {
      "scopeRedirects": true
    }
  }
}
```

It is opt-in because a redirect to a URL that is not allowed by the scope now fails with `url not allowed on the configured scope` instead of being followed, so applications that rely on being redirected outside of their scope must add the redirect target to the scope. **This will become the default in v3.**

Note that `tauri_plugin_http::init()` now returns `TauriPlugin<R, Option<Config>>` instead of `TauriPlugin<R>`.
