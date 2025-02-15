# Changelog

## \[2.2.1]

- [`ce11079f`](https://github.com/tauri-apps/plugins-workspace/commit/ce11079f19852fbefdecf0e4c7d947af3624fee0) ([#2280](https://github.com/tauri-apps/plugins-workspace/pull/2280) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Explicitly drop `arboard::Clipboard` on exit. Add recommendation to not use read methods on the mainthread.

## \[2.2.0]

- [`3a79266b`](https://github.com/tauri-apps/plugins-workspace/commit/3a79266b8cf96a55b1ae6339d725567d45a44b1d) ([#2173](https://github.com/tauri-apps/plugins-workspace/pull/2173) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Bumped all plugins to `v2.2.0`. From now, the versions for the Rust and JavaScript packages of each plugin will be in sync with each other.

## \[2.0.1]

- [`3fa0fc09`](https://github.com/tauri-apps/plugins-workspace/commit/3fa0fc09bbee0d619801e5757af9fb3c09883c97) ([#2099](https://github.com/tauri-apps/plugins-workspace/pull/2099) by [@rasteiner](https://github.com/tauri-apps/plugins-workspace/../../rasteiner)) Fix clipboard manager client side api not copying fallback alternative text when calling `writeHtml`.

## \[2.0.2]

- [`d57df4de`](https://github.com/tauri-apps/plugins-workspace/commit/d57df4debe7c75cfbd6d6558fff1beb07dbee54c) ([#1986](https://github.com/tauri-apps/plugins-workspace/pull/1986) by [@RikaKagurasaka](https://github.com/tauri-apps/plugins-workspace/../../RikaKagurasaka)) Fix that `read_image` wrongly set the image rgba data with binary PNG data.

## \[2.0.1]

- [`a1a82208`](https://github.com/tauri-apps/plugins-workspace/commit/a1a82208ed4ab87f83310be0dc95428aec9ab241) ([#1873](https://github.com/tauri-apps/plugins-workspace/pull/1873) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

## \[2.0.0]

- [`e2c4dfb6`](https://github.com/tauri-apps/plugins-workspace/commit/e2c4dfb6af43e5dd8d9ceba232c315f5febd55c1) Update to tauri v2 stable release.

## \[2.0.0-rc.2]

- [`341a5320`](https://github.com/tauri-apps/plugins-workspace/commit/341a5320c33d3c7b041abf7eb0ab7ad8009e6c3f) ([#1771](https://github.com/tauri-apps/plugins-workspace/pull/1771)) Fix warnings and clear implementation on Android below SDK 28.

## \[2.0.0-rc.1]

- [`e2e97db5`](https://github.com/tauri-apps/plugins-workspace/commit/e2e97db51983267f5be84d4f6f0278d58834d1f5) ([#1701](https://github.com/tauri-apps/plugins-workspace/pull/1701) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri 2.0.0-rc.8

## \[2.0.0-rc.2]

- [`b9147758`](https://github.com/tauri-apps/plugins-workspace/commit/b914775898c2bee7ceb20bd17ee595005cd17a64) ([#1679](https://github.com/tauri-apps/plugins-workspace/pull/1679) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Explicitly set a minimum macOS version for the Swift package.

## \[2.0.0-rc.1]

- [`2c00c029`](https://github.com/tauri-apps/plugins-workspace/commit/2c00c0292c9127b81567de46691e8c0f73557261) ([#1630](https://github.com/tauri-apps/plugins-workspace/pull/1630) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Fixed an issue that caused multi-word IIFE names to not be formatted correctly. For example the `barcode-scanner` was defined as `window.__TAURI_PLUGIN_CLIPBOARDMANAGER__` instead of `window.__TAURI_PLUGIN_CLIPBOARD_MANAGER__`.

### changes

- [`6b079cfd`](https://github.com/tauri-apps/plugins-workspace/commit/6b079cfdd107c94abc2c7300f6af00bac3ff4040) ([#1649](https://github.com/tauri-apps/plugins-workspace/pull/1649) by [@ahqsoftwares](https://github.com/tauri-apps/plugins-workspace/../../ahqsoftwares)) Remove targetSdk from build.kts files as it is deprecated and will be removed from DSL v9.0

## \[2.0.0-rc.0]

- [`9887d1`](https://github.com/tauri-apps/plugins-workspace/commit/9887d14bd0e971c4c0f5c1188fc4005d3fc2e29e) Update to tauri RC.

## \[2.1.0-beta.6]

- [`99d6ac0f`](https://github.com/tauri-apps/plugins-workspace/commit/99d6ac0f9506a6a4a1aa59c728157190a7441af6) ([#1606](https://github.com/tauri-apps/plugins-workspace/pull/1606) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) The JS packages now specify the *minimum* `@tauri-apps/api` version instead of a single exact version.
- [`6de87966`](https://github.com/tauri-apps/plugins-workspace/commit/6de87966ecc00ad9d91c25be452f1f46bd2b7e1f) ([#1597](https://github.com/tauri-apps/plugins-workspace/pull/1597) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Update to tauri beta.25.

## \[2.1.0-beta.5]

- [`22a17980`](https://github.com/tauri-apps/plugins-workspace/commit/22a17980ff4f6f8c40adb1b8f4ffc6dae2fe7e30) ([#1537](https://github.com/tauri-apps/plugins-workspace/pull/1537) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri beta.24.

## \[2.1.0-beta.4]

- [`76daee7a`](https://github.com/tauri-apps/plugins-workspace/commit/76daee7aafece34de3092c86e531cf9eb1138989) ([#1512](https://github.com/tauri-apps/plugins-workspace/pull/1512) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) Update to tauri beta.23.

## \[2.1.0-beta.3]

- [`9013854f`](https://github.com/tauri-apps/plugins-workspace/commit/9013854f42a49a230b9dbb9d02774765528a923f)([#1382](https://github.com/tauri-apps/plugins-workspace/pull/1382)) Update to tauri beta.22.

## \[2.1.0-beta.2]

- [`430bd6f4`](https://github.com/tauri-apps/plugins-workspace/commit/430bd6f4f379bee5d232ae6b098ae131db7f178a)([#1363](https://github.com/tauri-apps/plugins-workspace/pull/1363)) Update to tauri beta.20.

## \[2.1.0-beta.1]

- [`bd1ed590`](https://github.com/tauri-apps/plugins-workspace/commit/bd1ed5903ffcce5500310dac1e59e8c67674ef1e)([#1237](https://github.com/tauri-apps/plugins-workspace/pull/1237)) Update to tauri beta.17.

## \[2.1.0-beta.1]

- [`27b258c`](https://github.com/tauri-apps/plugins-workspace/commit/27b258cf31ae5557c99ae66537fb9196368d4d8b)([#1185](https://github.com/tauri-apps/plugins-workspace/pull/1185)) Expose `Clipboard` struct
- [`e3d41f4`](https://github.com/tauri-apps/plugins-workspace/commit/e3d41f4011bd3ea3ce281bb38bbe31d3709f8e0f)([#1191](https://github.com/tauri-apps/plugins-workspace/pull/1191)) Internally use the webview scoped resources table instead of the app one, so other webviews can't access other webviews resources.
- [`e3d41f4`](https://github.com/tauri-apps/plugins-workspace/commit/e3d41f4011bd3ea3ce281bb38bbe31d3709f8e0f)([#1191](https://github.com/tauri-apps/plugins-workspace/pull/1191)) Update for tauri 2.0.0-beta.15.

## \[2.1.0-beta.0]

- [`9dec960`](https://github.com/tauri-apps/plugins-workspace/commit/9dec9605ed1ce19dbef697e55debddf9008ecba1)([#845](https://github.com/tauri-apps/plugins-workspace/pull/845)) Add support for `read_image` and `write_image` to the clipboard plugin (desktop).

## \[2.0.0-beta.2]

- [`dc6d332`](https://github.com/tauri-apps/plugins-workspace/commit/dc6d3321e5305fa8b7250553bd179cbee995998a)([#977](https://github.com/tauri-apps/plugins-workspace/pull/977)) Add support for writing HTML content to the clipboard.
- [`99bea25`](https://github.com/tauri-apps/plugins-workspace/commit/99bea2559c2c0648c2519c50a18cd124dacef57b)([#1005](https://github.com/tauri-apps/plugins-workspace/pull/1005)) Update to tauri beta.8.

## \[2.0.0-beta.1]

- [`569defb`](https://github.com/tauri-apps/plugins-workspace/commit/569defbe9492e38938554bb7bdc1be9151456d21) Update to tauri beta.4.

## \[2.0.0-beta.0]

- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Update to tauri beta.
- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Add permissions.

## \[2.0.0-alpha.5]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.13.

## \[2.0.0-alpha.4]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.12.

## \[2.0.0-alpha.3]

- [`e438e0a`](https://github.com/tauri-apps/plugins-workspace/commit/e438e0a62d4b430a5159f05f13ecd397dd891a0d)([#676](https://github.com/tauri-apps/plugins-workspace/pull/676)) Update to @tauri-apps/api v2.0.0-alpha.11.

## \[2.0.0-alpha.2]

- [`5c13736`](https://github.com/tauri-apps/plugins-workspace/commit/5c137365c60790e8d4037d449e8237aa3fffdab0)([#673](https://github.com/tauri-apps/plugins-workspace/pull/673)) Update to @tauri-apps/api v2.0.0-alpha.9.

## \[2.0.0-alpha.2]

- [`4e2cef9`](https://github.com/tauri-apps/plugins-workspace/commit/4e2cef9b702bbbb9cf4ee17de50791cb21f1b2a4)([#593](https://github.com/tauri-apps/plugins-workspace/pull/593)) Update to alpha.12.

## \[2.0.0-alpha.1]

- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
