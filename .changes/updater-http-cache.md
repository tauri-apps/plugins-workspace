---
"updater": minor
"updater-js": minor
---

- Added an in-memory HTTP cache for update checks (downloads are not cached).
- New `plugins.updater.cache` config: `disabled`, `disabledByDefault` and `maxTtlMins` (defaults to 12 hours).
- The JS `check` command has a new `cacheMode` option (`default`, `bypass`, `checkNow`). In Rust this is exposed as `UpdaterBuilder::cache_mode_override`.
- **Behavior change:** Caching is opt-in in this release. You can opt-in with `check({cacheMode: 'checkNow'})`, `cacheMode: 'default'` or set `disabledByDefault = false`. Otherwise the updater behaves the same and transparently skips caching. This is intended as temporary and a future release is expected to change the default!
- **Rust API change:** Network errors from `check()` now come back as `Error::ReqwestMiddleware(Reqwest(..))` instead of `Error::Reqwest`, so code that matches on the old variant stops matching.
