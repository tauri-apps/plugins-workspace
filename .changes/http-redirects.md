---
http: minor
http-js: minor
---

Redirects are now followed by the plugin instead of `reqwest`, so the `requestHeaders` rules and the `on_request` hook apply again to each redirect target. Other changes to how redirects are followed:

- `Authorization`, `Cookie` and `Proxy-Authorization` headers set by the frontend are dropped once a redirect leaves the origin of the request, as the fetch spec requires. The origin includes the scheme, so this now also applies to a redirect from `http://` to `https://` on the same host, where `reqwest` used to keep them. Request such URLs over `https://` directly.
- Exceeding `maxRedirections` now fails with `too many redirects, the redirect to <url> was not followed` (the new `Error::TooManyRedirects`).
- `Error` is now `#[non_exhaustive]`, so a `match` on it needs a wildcard arm.
