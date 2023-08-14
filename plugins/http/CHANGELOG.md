# Changelog

## \[2.0.0-alpha.1]

- [`7d9df72`](https://github.com/tauri-apps/plugins-workspace/commit/7d9df7297a221a64d9de945ffc2cd8313d3104dc)([#428](https://github.com/tauri-apps/plugins-workspace/pull/428)) Multipart requests are now handled in JavaScript by the `Request` JavaScript class so you just need to use a `FormData` body and not set the content-type header to `multipart/form-data`. `application/x-www-form-urlencoded` requests must be done manually.
- [`7d9df72`](https://github.com/tauri-apps/plugins-workspace/commit/7d9df7297a221a64d9de945ffc2cd8313d3104dc)([#428](https://github.com/tauri-apps/plugins-workspace/pull/428)) The http plugin has been rewritten from scratch and now only exposes a `fetch` function in Javascript and Re-exports `reqwest` crate in Rust. The new `fetch` method tries to be as close and compliant to the `fetch` Web API as possible.
- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
