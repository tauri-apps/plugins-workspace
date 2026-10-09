---
fs: patch
fs-js: patch
---

**Security:** the `open` command no longer accepts a `customFlags` option from the webview. It was not part of the JavaScript types and let the webview pass arbitrary `open(2)` flags (e.g. `O_TRUNC`) to files it may only read. `OpenOptionsExt::custom_flags` still works from Rust.
