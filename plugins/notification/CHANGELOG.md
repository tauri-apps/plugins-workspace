# Changelog

## \[2.0.0-rc.2]

- [`b9147758`](https://github.com/tauri-apps/plugins-workspace/commit/b914775898c2bee7ceb20bd17ee595005cd17a64) ([#1679](https://github.com/tauri-apps/plugins-workspace/pull/1679) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Explicitly set a minimum macOS version for the Swift package.

## \[2.0.0-rc.1]

### changes

- [`6b079cfd`](https://github.com/tauri-apps/plugins-workspace/commit/6b079cfdd107c94abc2c7300f6af00bac3ff4040) ([#1649](https://github.com/tauri-apps/plugins-workspace/pull/1649) by [@ahqsoftwares](https://github.com/tauri-apps/plugins-workspace/../../ahqsoftwares)) Remove targetSdk from build.kts files as it is deprecated and will be removed from DSL v9.0

## \[2.0.0-rc.0]

- [`9887d1`](https://github.com/tauri-apps/plugins-workspace/commit/9887d14bd0e971c4c0f5c1188fc4005d3fc2e29e) Update to tauri RC.

## \[2.0.0-beta.8]

- [`99d6ac0f`](https://github.com/tauri-apps/plugins-workspace/commit/99d6ac0f9506a6a4a1aa59c728157190a7441af6) ([#1606](https://github.com/tauri-apps/plugins-workspace/pull/1606) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) The JS packages now specify the *minimum* `@tauri-apps/api` version instead of a single exact version.
- [`6de87966`](https://github.com/tauri-apps/plugins-workspace/commit/6de87966ecc00ad9d91c25be452f1f46bd2b7e1f) ([#1597](https://github.com/tauri-apps/plugins-workspace/pull/1597) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Update to tauri beta.25.

## \[2.0.0-beta.11]

- [`725ff429`](https://github.com/tauri-apps/plugins-workspace/commit/725ff4295e56df9c30c099813bd64b96fe61b945) ([#1556](https://github.com/tauri-apps/plugins-workspace/pull/1556) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Fixed an issue that caused the `notification` plugin's initialization script to cause the WebView on Windows to throw a `STATUS_ACCESS_VIOLATION` error on remote websites.

## \[2.0.0-beta.7]

- [`22a17980`](https://github.com/tauri-apps/plugins-workspace/commit/22a17980ff4f6f8c40adb1b8f4ffc6dae2fe7e30) ([#1537](https://github.com/tauri-apps/plugins-workspace/pull/1537) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri beta.24.

## \[2.0.0-beta.6]

- [`76daee7a`](https://github.com/tauri-apps/plugins-workspace/commit/76daee7aafece34de3092c86e531cf9eb1138989) ([#1512](https://github.com/tauri-apps/plugins-workspace/pull/1512) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) Update to tauri beta.23.

## \[2.0.0-beta.8]

- [`3779fb50`](https://github.com/tauri-apps/plugins-workspace/commit/3779fb50634fba4d7e7eb0bfecc2216349b9d64d) ([#1432](https://github.com/tauri-apps/plugins-workspace/pull/1432) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Use notify_rust from crates.io instead of local fork.

## \[2.0.0-beta.5]

- [`9013854f`](https://github.com/tauri-apps/plugins-workspace/commit/9013854f42a49a230b9dbb9d02774765528a923f)([#1382](https://github.com/tauri-apps/plugins-workspace/pull/1382)) Update to tauri beta.22.

## \[2.0.0-beta.4]

- [`430bd6f4`](https://github.com/tauri-apps/plugins-workspace/commit/430bd6f4f379bee5d232ae6b098ae131db7f178a)([#1363](https://github.com/tauri-apps/plugins-workspace/pull/1363)) Update to tauri beta.20.

## \[2.0.0-beta.3]

- [`bd1ed590`](https://github.com/tauri-apps/plugins-workspace/commit/bd1ed5903ffcce5500310dac1e59e8c67674ef1e)([#1237](https://github.com/tauri-apps/plugins-workspace/pull/1237)) Update to tauri beta.17.

## \[2.0.0-beta.4]

- [`326df688`](https://github.com/tauri-apps/plugins-workspace/commit/326df6883998d416fc0837583ed972854628bb52)([#1236](https://github.com/tauri-apps/plugins-workspace/pull/1236)) Fixes command argument parsing on iOS.

## \[2.0.0-beta.3]

- [`a04ea2f`](https://github.com/tauri-apps/plugins-workspace/commit/a04ea2f38294d5a3987578283badc8eec87a7752)([#1071](https://github.com/tauri-apps/plugins-workspace/pull/1071)) The global API script is now only added to the binary when the `withGlobalTauri` config is true.
- [`62ce5df`](https://github.com/tauri-apps/plugins-workspace/commit/62ce5df52ca3c86786e711ef193a206e7b0dc0cf)([#1096](https://github.com/tauri-apps/plugins-workspace/pull/1096)) Fix development mode check to set the app ID on macOS.

## \[2.0.0-beta.2]

- [`99bea25`](https://github.com/tauri-apps/plugins-workspace/commit/99bea2559c2c0648c2519c50a18cd124dacef57b)([#1005](https://github.com/tauri-apps/plugins-workspace/pull/1005)) Update to tauri beta.8.

## \[2.0.0-beta.1]

- [`569defb`](https://github.com/tauri-apps/plugins-workspace/commit/569defbe9492e38938554bb7bdc1be9151456d21) Update to tauri beta.4.

## \[2.0.0-beta.0]

- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Update to tauri beta.
- [`1b1d795`](https://github.com/tauri-apps/plugins-workspace/commit/1b1d795b5866e5524a9a9925f0fb7b2f8e3e3675)([#874](https://github.com/tauri-apps/plugins-workspace/pull/874)) Export the missing `Schedule` class.
- [`8dea78a`](https://github.com/tauri-apps/plugins-workspace/commit/8dea78ac7dcb502159e66bad464094696aa257d4)([#909](https://github.com/tauri-apps/plugins-workspace/pull/909)) Fixes deserialization and implementation bugs with scheduled notifications on Android.

## \[2.0.0-alpha.5]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.13.

## \[2.0.0-alpha.4]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.12.

## \[2.0.0-alpha.3]

- [`e438e0a`](https://github.com/tauri-apps/plugins-workspace/commit/e438e0a62d4b430a5159f05f13ecd397dd891a0d)([#676](https://github.com/tauri-apps/plugins-workspace/pull/676)) Update to @tauri-apps/api v2.0.0-alpha.11.

## \[2.0.0-alpha.2]

- [`5c13736`](https://github.com/tauri-apps/plugins-workspace/commit/5c137365c60790e8d4037d449e8237aa3fffdab0)([#673](https://github.com/tauri-apps/plugins-workspace/pull/673)) Update to @tauri-apps/api v2.0.0-alpha.9.

## \[2.0.0-alpha.3]

- [`4e2cef9`](https://github.com/tauri-apps/plugins-workspace/commit/4e2cef9b702bbbb9cf4ee17de50791cb21f1b2a4)([#593](https://github.com/tauri-apps/plugins-workspace/pull/593)) Update to alpha.12.

## \[2.0.0-alpha.1]

- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

## \[2.0.0-alpha.1]

- [`d8b4aca`](https://github.com/tauri-apps/plugins-workspace/commit/d8b4aca69f628b170804ecb982e2c319d026ef47)([#414](https://github.com/tauri-apps/plugins-workspace/pull/414)) Use `window.__TAURI_INVOKE__` instead of `window.__TAURI__` in init.js, fixes usage in apps without `withGlobalTauri` enabled.
- [`7d71ad4`](https://github.com/tauri-apps/plugins-workspace/commit/7d71ad4e587bcf47ea34645f5b226945e487b765) Play a default sound when showing a notification on Windows.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  ub.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

## \[2.0.0-alpha.1]

- [`d8b4aca`](https://github.com/tauri-apps/plugins-workspace/commit/d8b4aca69f628b170804ecb982e2c319d026ef47)([#414](https://github.com/tauri-apps/plugins-workspace/pull/414)) Use `window.__TAURI_INVOKE__` instead of `window.__TAURI__` in init.js, fixes usage in apps without `withGlobalTauri` enabled.
- [`7d71ad4`](https://github.com/tauri-apps/plugins-workspace/commit/7d71ad4e587bcf47ea34645f5b226945e487b765) Play a default sound when showing a notification on Windows.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  ithub.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  ithub.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  !
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  /commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  !
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
