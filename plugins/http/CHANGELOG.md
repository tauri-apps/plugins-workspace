# Changelog

## \[2.0.0-beta.3]

- [`bd1ed590`](https://github.com/tauri-apps/plugins-workspace/commit/bd1ed5903ffcce5500310dac1e59e8c67674ef1e)([#1237](https://github.com/tauri-apps/plugins-workspace/pull/1237)) Update to tauri beta.17.

## \[2.0.0-beta.6]

### Dependencies

- Upgraded to `fs@2.0.0-beta.6`

## \[2.0.0-beta.5]

- [`500ff10`](https://github.com/tauri-apps/plugins-workspace/commit/500ff10fbd89fdfc73caf9d153029dad567b4ff1)([#1166](https://github.com/tauri-apps/plugins-workspace/pull/1166)) **Breaking change:** Removed the `default-tls` feature flag. The `rustls-tls`, `http2`, `macos-system-configuration`, and `charset` feature flags are now enabled by default.
- [`e3d41f4`](https://github.com/tauri-apps/plugins-workspace/commit/e3d41f4011bd3ea3ce281bb38bbe31d3709f8e0f)([#1191](https://github.com/tauri-apps/plugins-workspace/pull/1191)) Internally use the webview scoped resources table instead of the app one, so other webviews can't access other webviews resources.
- [`7e2fcc5`](https://github.com/tauri-apps/plugins-workspace/commit/7e2fcc5e74df7c3c718e40f75bfb0eafc7d69d8d)([#1146](https://github.com/tauri-apps/plugins-workspace/pull/1146)) Update dependencies to align with tauri 2.0.0-beta.14.
- [`e3d41f4`](https://github.com/tauri-apps/plugins-workspace/commit/e3d41f4011bd3ea3ce281bb38bbe31d3709f8e0f)([#1191](https://github.com/tauri-apps/plugins-workspace/pull/1191)) Update for tauri 2.0.0-beta.15.

### Dependencies

- Upgraded to `fs@2.0.0-beta.5`

## \[2.0.0-beta.4]

### Dependencies

- Upgraded to `fs@2.0.0-beta.4`

## \[2.0.0-beta.3]

- [`c873e4d`](https://github.com/tauri-apps/plugins-workspace/commit/c873e4d6c74e759742f7c9a88e35cff10a75122a)([#1059](https://github.com/tauri-apps/plugins-workspace/pull/1059)) Fixes scope not allowing subpaths, query parameters and hash when those values are empty.
- [`a04ea2f`](https://github.com/tauri-apps/plugins-workspace/commit/a04ea2f38294d5a3987578283badc8eec87a7752)([#1071](https://github.com/tauri-apps/plugins-workspace/pull/1071)) The global API script is now only added to the binary when the `withGlobalTauri` config is true.
- [`753c7be`](https://github.com/tauri-apps/plugins-workspace/commit/753c7be0a6a78121d2e88ea0efc3040580c885b4)([#1050](https://github.com/tauri-apps/plugins-workspace/pull/1050)) Add `unsafe-headers` cargo feature flag to allow using [forbidden headers](https://fetch.spec.whatwg.org/#terminology-headers).

### Dependencies

- Upgraded to `fs@2.0.0-beta.3`

## \[2.0.0-beta.2]

- [`ae56b13`](https://github.com/tauri-apps/plugins-workspace/commit/ae56b13a4d49dbf922b8a0fbb0d557bb63c1d72b)([#983](https://github.com/tauri-apps/plugins-workspace/pull/983)) Allow `User-Agent` header to be set.
- [`99bea25`](https://github.com/tauri-apps/plugins-workspace/commit/99bea2559c2c0648c2519c50a18cd124dacef57b)([#1005](https://github.com/tauri-apps/plugins-workspace/pull/1005)) Update to tauri beta.8.

## \[2.0.0-beta.1]

- [`569defb`](https://github.com/tauri-apps/plugins-workspace/commit/569defbe9492e38938554bb7bdc1be9151456d21) Update to tauri beta.4.

## \[2.0.0-beta.0]

- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Update to tauri beta.
- [`1a34720`](https://github.com/tauri-apps/plugins-workspace/commit/1a347203a54eccc954749d11c4ee81fdd9a0cde7)([#858](https://github.com/tauri-apps/plugins-workspace/pull/858)) Fix http fetch client option init with parameter `connectTimeout`

## \[2.0.0-alpha.9]

### Dependencies

- Upgraded to `fs@2.0.0-alpha.7`

## \[2.0.0-alpha.6]

- [`bfa87da`](https://github.com/tauri-apps/plugins-workspace/commit/bfa87da848f9f1da2abae3354eed632881eddf11)([#824](https://github.com/tauri-apps/plugins-workspace/pull/824)) Add `proxy` field to `fetch` options to configure proxy.

## \[2.0.0-alpha.5]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.13.

## \[2.0.0-alpha.4]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.12.

## \[2.0.0-alpha.3]

- [`e438e0a`](https://github.com/tauri-apps/plugins-workspace/commit/e438e0a62d4b430a5159f05f13ecd397dd891a0d)([#676](https://github.com/tauri-apps/plugins-workspace/pull/676)) Update to @tauri-apps/api v2.0.0-alpha.11.

## \[2.0.0-alpha.2]

- [`5c13736`](https://github.com/tauri-apps/plugins-workspace/commit/5c137365c60790e8d4037d449e8237aa3fffdab0)([#673](https://github.com/tauri-apps/plugins-workspace/pull/673)) Update to @tauri-apps/api v2.0.0-alpha.9.

## \[2.0.0-alpha.3]

- [`2cb0fa7`](https://github.com/tauri-apps/plugins-workspace/commit/2cb0fa719b8b1f5ac07dada93520dbbcf637d64c)([#587](https://github.com/tauri-apps/plugins-workspace/pull/587)) Remove `cmd` property which breaks invoke call.
- [`4e2cef9`](https://github.com/tauri-apps/plugins-workspace/commit/4e2cef9b702bbbb9cf4ee17de50791cb21f1b2a4)([#593](https://github.com/tauri-apps/plugins-workspace/pull/593)) Update to alpha.12.

### Dependencies

- Upgraded to `fs@2.0.0-alpha.2`

## \[2.0.0-alpha.2]

- [`aec17a9`](https://github.com/tauri-apps/plugins-workspace/commit/aec17a90fc365774c70c4876b94a899416120e26)([#558](https://github.com/tauri-apps/plugins-workspace/pull/558)) Improve response performance by using the new IPC streaming data.

## \[2.0.0-alpha.1]

- [`7d9df72`](https://github.com/tauri-apps/plugins-workspace/commit/7d9df7297a221a64d9de945ffc2cd8313d3104dc)([#428](https://github.com/tauri-apps/plugins-workspace/pull/428)) Multipart requests are now handled in JavaScript by the `Request` JavaScript class so you just need to use a `FormData` body and not set the content-type header to `multipart/form-data`. `application/x-www-form-urlencoded` requests must be done manually.
- [`7d9df72`](https://github.com/tauri-apps/plugins-workspace/commit/7d9df7297a221a64d9de945ffc2cd8313d3104dc)([#428](https://github.com/tauri-apps/plugins-workspace/pull/428)) The http plugin has been rewritten from scratch and now only exposes a `fetch` function in Javascript and Re-exports `reqwest` crate in Rust. The new `fetch` method tries to be as close and compliant to the `fetch` Web API as possible.
- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  /717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  ace/pull/371)) First v2 alpha release!
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  2 alpha release!
  !
  371\)) First v2 alpha release!
  /717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  ace/pull/371)) First v2 alpha release!
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
  !
  371\)) First v2 alpha release!
lpha release!
  !
  371\)) First v2 alpha release!
