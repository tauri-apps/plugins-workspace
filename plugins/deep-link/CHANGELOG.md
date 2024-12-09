# Changelog

## \[2.2.0]

- [`3a79266b`](https://github.com/tauri-apps/plugins-workspace/commit/3a79266b8cf96a55b1ae6339d725567d45a44b1d) ([#2173](https://github.com/tauri-apps/plugins-workspace/pull/2173) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Bumped all plugins to `v2.2.0`. From now, the versions for the Rust and JavaScript packages of each plugin will be in sync with each other.

## \[2.0.1]

- [`b2aea045`](https://github.com/tauri-apps/plugins-workspace/commit/b2aea0456799775a7243706fdd7a5abf9a193992) ([#2008](https://github.com/tauri-apps/plugins-workspace/pull/2008) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) `onOpenUrl()` will now not call `getCurrent()` anymore, matching the documented behavior.

## \[2.0.1]

- [`a1a82208`](https://github.com/tauri-apps/plugins-workspace/commit/a1a82208ed4ab87f83310be0dc95428aec9ab241) ([#1873](https://github.com/tauri-apps/plugins-workspace/pull/1873) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

## \[2.0.0]

- [`e2c4dfb6`](https://github.com/tauri-apps/plugins-workspace/commit/e2c4dfb6af43e5dd8d9ceba232c315f5febd55c1) Update to tauri v2 stable release.

## \[2.0.0-rc.7]

- [`3168e176`](https://github.com/tauri-apps/plugins-workspace/commit/3168e176031a61215be542595ba90ca51f8f2d97) ([#1806](https://github.com/tauri-apps/plugins-workspace/pull/1806) by [@auggiebennett](https://github.com/tauri-apps/plugins-workspace/../../auggiebennett)) Fix fails to start when having spaces in the main binary path on Windows

## \[2.0.0-rc.6]

- [`6f3f6679`](https://github.com/tauri-apps/plugins-workspace/commit/6f3f66794a87ef9d1c16667c425d5ad7091a9c2f) ([#1780](https://github.com/tauri-apps/plugins-workspace/pull/1780)) Added `DeepLink::on_open_url` function to match the JavaScript API implementation,
  which wraps the `deep-link://new-url` event and also send the current deep link if there's any.

## \[2.0.0-rc.5]

- [`984110a9`](https://github.com/tauri-apps/plugins-workspace/commit/984110a978774712bad4d746ed06134d54debcd0) ([#1770](https://github.com/tauri-apps/plugins-workspace/pull/1770) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Emit the `deep-link://new-url` event on Linux and Windows when the app is executed with a deep link CLI argument,
  matching the iOS and macOS behavior.

## \[2.0.0-rc.2]

- [`64a6240f`](https://github.com/tauri-apps/plugins-workspace/commit/64a6240f79fcd52267c8d721b727ae695055d7ff) ([#1759](https://github.com/tauri-apps/plugins-workspace/pull/1759) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Implement `get_current` on Linux and Windows.

## \[2.0.0-rc.3]

- [`4654591d`](https://github.com/tauri-apps/plugins-workspace/commit/4654591d820403d6fa1a007fd55bb0d85947a6cc) ([#1732](https://github.com/tauri-apps/plugins-workspace/pull/1732) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Allow empty configuration values.

## \[2.0.0-rc.1]

- [`e2e97db5`](https://github.com/tauri-apps/plugins-workspace/commit/e2e97db51983267f5be84d4f6f0278d58834d1f5) ([#1701](https://github.com/tauri-apps/plugins-workspace/pull/1701) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri 2.0.0-rc.8

## \[2.0.0-rc.1]

- [`2c00c029`](https://github.com/tauri-apps/plugins-workspace/commit/2c00c0292c9127b81567de46691e8c0f73557261) ([#1630](https://github.com/tauri-apps/plugins-workspace/pull/1630) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Fixed an issue that caused multi-word IIFE names to not be formatted correctly. For example the `barcode-scanner` was defined as `window.__TAURI_PLUGIN_CLIPBOARDMANAGER__` instead of `window.__TAURI_PLUGIN_CLIPBOARD_MANAGER__`.
- [`5d170a54`](https://github.com/tauri-apps/plugins-workspace/commit/5d170a5444982dcc14135f6f1fc3e5da359f0eb0) ([#1671](https://github.com/tauri-apps/plugins-workspace/pull/1671) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri 2.0.0-rc.3.

### changes

- [`6b079cfd`](https://github.com/tauri-apps/plugins-workspace/commit/6b079cfdd107c94abc2c7300f6af00bac3ff4040) ([#1649](https://github.com/tauri-apps/plugins-workspace/pull/1649) by [@ahqsoftwares](https://github.com/tauri-apps/plugins-workspace/../../ahqsoftwares)) Remove targetSdk from build.kts files as it is deprecated and will be removed from DSL v9.0

## \[2.0.0-rc.0]

- [`9887d1`](https://github.com/tauri-apps/plugins-workspace/commit/9887d14bd0e971c4c0f5c1188fc4005d3fc2e29e) Update to tauri RC.

## \[2.0.0-beta.10]

- [`99d6ac0f`](https://github.com/tauri-apps/plugins-workspace/commit/99d6ac0f9506a6a4a1aa59c728157190a7441af6) ([#1606](https://github.com/tauri-apps/plugins-workspace/pull/1606) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) The JS packages now specify the *minimum* `@tauri-apps/api` version instead of a single exact version.
- [`6de87966`](https://github.com/tauri-apps/plugins-workspace/commit/6de87966ecc00ad9d91c25be452f1f46bd2b7e1f) ([#1597](https://github.com/tauri-apps/plugins-workspace/pull/1597) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Update to tauri beta.25.

## \[2.0.0-beta.9]

- [`22a17980`](https://github.com/tauri-apps/plugins-workspace/commit/22a17980ff4f6f8c40adb1b8f4ffc6dae2fe7e30) ([#1537](https://github.com/tauri-apps/plugins-workspace/pull/1537) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri beta.24.

## \[2.0.0-beta.8]

- [`76daee7a`](https://github.com/tauri-apps/plugins-workspace/commit/76daee7aafece34de3092c86e531cf9eb1138989) ([#1512](https://github.com/tauri-apps/plugins-workspace/pull/1512) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) Update to tauri beta.23.

## \[2.0.0-beta.7]

- [`0b008882`](https://github.com/tauri-apps/plugins-workspace/commit/0b0088821e50e33825f7d573b1c826cfeb38dda0) ([#1404](https://github.com/tauri-apps/plugins-workspace/pull/1404) by [@simonhyll](https://github.com/tauri-apps/plugins-workspace/../../simonhyll)) Fixed a typo in the `deep-link` js bindings causing `isRegistered` to not work.

## \[2.0.0-beta.6]

- [`9013854f`](https://github.com/tauri-apps/plugins-workspace/commit/9013854f42a49a230b9dbb9d02774765528a923f)([#1382](https://github.com/tauri-apps/plugins-workspace/pull/1382)) Update to tauri beta.22.

## \[2.0.0-beta.5]

- [`430bd6f4`](https://github.com/tauri-apps/plugins-workspace/commit/430bd6f4f379bee5d232ae6b098ae131db7f178a)([#1363](https://github.com/tauri-apps/plugins-workspace/pull/1363)) Update to tauri beta.20.

## \[2.0.0-beta.4]

- [`021d23be`](https://github.com/tauri-apps/plugins-workspace/commit/021d23bef330de4ce001993e0ef2c7ab7815f044)([#916](https://github.com/tauri-apps/plugins-workspace/pull/916)) Added desktop support.

## \[2.0.0-beta.3]

- [`bd1ed590`](https://github.com/tauri-apps/plugins-workspace/commit/bd1ed5903ffcce5500310dac1e59e8c67674ef1e)([#1237](https://github.com/tauri-apps/plugins-workspace/pull/1237)) Update to tauri beta.17.

## \[2.0.0-beta.3]

- [`a04ea2f`](https://github.com/tauri-apps/plugins-workspace/commit/a04ea2f38294d5a3987578283badc8eec87a7752)([#1071](https://github.com/tauri-apps/plugins-workspace/pull/1071)) The global API script is now only added to the binary when the `withGlobalTauri` config is true.

## \[2.0.0-beta.2]

- [`99bea25`](https://github.com/tauri-apps/plugins-workspace/commit/99bea2559c2c0648c2519c50a18cd124dacef57b)([#1005](https://github.com/tauri-apps/plugins-workspace/pull/1005)) Update to tauri beta.8.

## \[2.0.0-beta.1]

- [`569defb`](https://github.com/tauri-apps/plugins-workspace/commit/569defbe9492e38938554bb7bdc1be9151456d21) Update to tauri beta.4.

## \[2.0.0-beta.0]

- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Update to tauri beta.

## \[2.0.0-alpha.5]

- [`8b1d821`](https://github.com/tauri-apps/plugins-workspace/commit/8b1d821a375d66a61e06c78b7148e255855cfe1b)([#844](https://github.com/tauri-apps/plugins-workspace/pull/844)) Fixes issue with tauri alpha.20.

## \[2.0.0-alpha.4]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.13.

## \[2.0.0-alpha.3]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.12.

## \[2.0.0-alpha.2]

- [`e438e0a`](https://github.com/tauri-apps/plugins-workspace/commit/e438e0a62d4b430a5159f05f13ecd397dd891a0d)([#676](https://github.com/tauri-apps/plugins-workspace/pull/676)) Update to @tauri-apps/api v2.0.0-alpha.11.

## \[2.0.0-alpha.1]

- [`5c13736`](https://github.com/tauri-apps/plugins-workspace/commit/5c137365c60790e8d4037d449e8237aa3fffdab0)([#673](https://github.com/tauri-apps/plugins-workspace/pull/673)) Update to @tauri-apps/api v2.0.0-alpha.9.

## \[2.0.0-alpha.0]

- [`eccd6f9`](https://github.com/tauri-apps/plugins-workspace/commit/eccd6f977af7629255b6f5a5205666c9079a86ed)([#504](https://github.com/tauri-apps/plugins-workspace/pull/504)) Initial release.
  0.0-alpha.0]

- [`eccd6f9`](https://github.com/tauri-apps/plugins-workspace/commit/eccd6f977af7629255b6f5a5205666c9079a86ed)([#504](https://github.com/tauri-apps/plugins-workspace/pull/504)) Initial release.

- [`eccd6f9`](https://github.com/tauri-apps/plugins-workspace/commit/eccd6f977af7629255b6f5a5205666c9079a86ed)([#504](https://github.com/tauri-apps/plugins-workspace/pull/504)) Initial release.
  commit/eccd6f977af7629255b6f5a5205666c9079a86ed)([#504](https://github.com/tauri-apps/plugins-workspace/pull/504)) Initial release.
  om/tauri-apps/plugins-workspace/commit/eccd6f977af7629255b6f5a5205666c9079a86ed)([#504](https://github.com/tauri-apps/plugins-workspace/pull/504)) Initial release.

- [`eccd6f9`](https://github.com/tauri-apps/plugins-workspace/commit/eccd6f977af7629255b6f5a5205666c9079a86ed)([#504](https://github.com/tauri-apps/plugins-workspace/pull/504)) Initial release.
  commit/eccd6f977af7629255b6f5a5205666c9079a86ed)([#504](https://github.com/tauri-apps/plugins-workspace/pull/504)) Initial release.
  ithub.com/tauri-apps/plugins-workspace/pull/504)) Initial release.
  ]\(https://github.com/tauri-apps/plugins-workspace/commit/eccd6f977af7629255b6f5a5205666c9079a86ed)([#504](https://github.com/tauri-apps/plugins-workspace/pull/504)) Initial release.
  commit/eccd6f977af7629255b6f5a5205666c9079a86ed)([#504](https://github.com/tauri-apps/plugins-workspace/pull/504)) Initial release.
  ithub.com/tauri-apps/plugins-workspace/pull/504)) Initial release.
