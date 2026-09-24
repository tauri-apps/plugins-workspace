---
"updater": minor
"updater-js": minor
---

- Added an in-memory HTTP cache for update checks (downloads are not cached).
- New `plugins.updater.cache` config: `disabled`, `disabledByDefault` and `maxTtlMins` (defaults to 12 hours).
- The JS `check` command has a new `cacheMode` option (`default`, `bypass`, `checkNow`). In Rust this is exposed as `UpdaterBuilder::cache_mode_override`.
- Raised MSRV to 1.89.0.
- **Behavior change:** Caching is enabled by default. Responses are cached for 1 hour when servers don't set `Cache-Control: max-age` or `Expires` headers, and never longer than `maxTtlMins` (default 12h). Incorrectly configured servers which don't set appropriate `Vary` headers can cause stale responses when the response should change from request headers.
- **Behavior change:** Using `check()` defaults to using cache. Which may be unexpected for a user facing "check now" button. Recommended to explicitly set there `check({cacheMode: 'checkNow'})`.
- **Rust API change:** Network errors from `check()` now come back as `Error::ReqwestMiddleware(Reqwest(..))` instead of `Error::Reqwest`, so code that matches on the old variant stops matching.
