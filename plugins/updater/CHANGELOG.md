# Changelog

## \[2.0.0-rc.0]

- [`9887d1`](https://github.com/tauri-apps/plugins-workspace/commit/9887d14bd0e971c4c0f5c1188fc4005d3fc2e29e) Update to tauri RC.

## \[2.0.0-beta.8]

- [`99d6ac0f`](https://github.com/tauri-apps/plugins-workspace/commit/99d6ac0f9506a6a4a1aa59c728157190a7441af6) ([#1606](https://github.com/tauri-apps/plugins-workspace/pull/1606) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) The JS packages now specify the *minimum* `@tauri-apps/api` version instead of a single exact version.
- [`6de87966`](https://github.com/tauri-apps/plugins-workspace/commit/6de87966ecc00ad9d91c25be452f1f46bd2b7e1f) ([#1597](https://github.com/tauri-apps/plugins-workspace/pull/1597) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Update to tauri beta.25.

## \[2.0.0-beta.11]

- [`f83b9e98`](https://github.com/tauri-apps/plugins-workspace/commit/f83b9e9813843df19b03b6af1018d848111b2a62) ([#1544](https://github.com/tauri-apps/plugins-workspace/pull/1544) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) On Windows, use a named tempfile with `<app name>-<version>-installer.exe` (or `.msi`) for v2 updater

  **Breaking Change**: `UpdaterBuilder::new` now takes one more argument `app_name: String`

## \[2.0.0-beta.7]

- [`22a17980`](https://github.com/tauri-apps/plugins-workspace/commit/22a17980ff4f6f8c40adb1b8f4ffc6dae2fe7e30) ([#1537](https://github.com/tauri-apps/plugins-workspace/pull/1537) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri beta.24.

## \[2.0.0-beta.6]

- [`76daee7a`](https://github.com/tauri-apps/plugins-workspace/commit/76daee7aafece34de3092c86e531cf9eb1138989) ([#1512](https://github.com/tauri-apps/plugins-workspace/pull/1512) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) Update to tauri beta.23.

## \[2.0.0-beta.8]

- [`bf29a72b`](https://github.com/tauri-apps/plugins-workspace/commit/bf29a72baaff15214a21989df23081eee84e3b8b) ([#1454](https://github.com/tauri-apps/plugins-workspace/pull/1454) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Fix regression in updater plugin failing to update using `.msi` installer.

## \[2.0.0-beta.5]

- [`9013854f`](https://github.com/tauri-apps/plugins-workspace/commit/9013854f42a49a230b9dbb9d02774765528a923f)([#1382](https://github.com/tauri-apps/plugins-workspace/pull/1382)) Update to tauri beta.22.

## \[2.0.0-beta.4]

- [`430bd6f4`](https://github.com/tauri-apps/plugins-workspace/commit/430bd6f4f379bee5d232ae6b098ae131db7f178a)([#1363](https://github.com/tauri-apps/plugins-workspace/pull/1363)) Update to tauri beta.20.
- [`43224c5d`](https://github.com/tauri-apps/plugins-workspace/commit/43224c5d5cfe2dd676e79ebafe424027c62c51c3)([#1330](https://github.com/tauri-apps/plugins-workspace/pull/1330)) Add `Update.download` and `Update.install` functions to the JavaScript API

## \[2.0.0-beta.3]

- [`bd1ed590`](https://github.com/tauri-apps/plugins-workspace/commit/bd1ed5903ffcce5500310dac1e59e8c67674ef1e)([#1237](https://github.com/tauri-apps/plugins-workspace/pull/1237)) Update to tauri beta.17.

## \[2.0.0-beta.4]

- [`293f363`](https://github.com/tauri-apps/plugins-workspace/commit/293f363c0dccc43e8403729fdc8cc2b4311c2d5b)([#1175](https://github.com/tauri-apps/plugins-workspace/pull/1175)) Add a `on_before_exit` hook for cleanup before spawning the updater on Windows, defaults to `app.cleanup_before_exit` when used through `UpdaterExt`
- [`293f363`](https://github.com/tauri-apps/plugins-workspace/commit/293f363c0dccc43e8403729fdc8cc2b4311c2d5b)([#1175](https://github.com/tauri-apps/plugins-workspace/pull/1175)) **Breaking change:** The `rustls-tls` feature flag is now enabled by default.
- [`e3d41f4`](https://github.com/tauri-apps/plugins-workspace/commit/e3d41f4011bd3ea3ce281bb38bbe31d3709f8e0f)([#1191](https://github.com/tauri-apps/plugins-workspace/pull/1191)) Internally use the webview scoped resources table instead of the app one, so other webviews can't access other webviews resources.
- [`7e2fcc5`](https://github.com/tauri-apps/plugins-workspace/commit/7e2fcc5e74df7c3c718e40f75bfb0eafc7d69d8d)([#1146](https://github.com/tauri-apps/plugins-workspace/pull/1146)) Update dependencies to align with tauri 2.0.0-beta.14.
- [`e3d41f4`](https://github.com/tauri-apps/plugins-workspace/commit/e3d41f4011bd3ea3ce281bb38bbe31d3709f8e0f)([#1191](https://github.com/tauri-apps/plugins-workspace/pull/1191)) Update for tauri 2.0.0-beta.15.

## \[2.0.0-beta.3]

- [`4e37316`](https://github.com/tauri-apps/plugins-workspace/commit/4e37316af0d6532bf9a9bd0e712b5b14b0598285)([#1051](https://github.com/tauri-apps/plugins-workspace/pull/1051)) Fix deserialization of `windows > installerArgs` config field.
- [`4e37316`](https://github.com/tauri-apps/plugins-workspace/commit/4e37316af0d6532bf9a9bd0e712b5b14b0598285)([#1051](https://github.com/tauri-apps/plugins-workspace/pull/1051)) On Windows, fallback to `passive` install mode when not defined in config.
- [`a3b5396`](https://github.com/tauri-apps/plugins-workspace/commit/a3b5396113ca93912274f6890d9ef5b1a409587a)([#1054](https://github.com/tauri-apps/plugins-workspace/pull/1054)) Fix Windows powershell window flashing on update
- [`a04ea2f`](https://github.com/tauri-apps/plugins-workspace/commit/a04ea2f38294d5a3987578283badc8eec87a7752)([#1071](https://github.com/tauri-apps/plugins-workspace/pull/1071)) The global API script is now only added to the binary when the `withGlobalTauri` config is true.

## \[2.0.0-beta.2]

- [`99bea25`](https://github.com/tauri-apps/plugins-workspace/commit/99bea2559c2c0648c2519c50a18cd124dacef57b)([#1005](https://github.com/tauri-apps/plugins-workspace/pull/1005)) Update to tauri beta.8.

## \[2.0.0-beta.1]

- [`569defb`](https://github.com/tauri-apps/plugins-workspace/commit/569defbe9492e38938554bb7bdc1be9151456d21) Update to tauri beta.4.

## \[2.0.0-beta.0]

- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Update to tauri beta.
- [`0879a87`](https://github.com/tauri-apps/plugins-workspace/commit/0879a87a7ecc83c9e886e6f1412fe253082b8d34)([#899](https://github.com/tauri-apps/plugins-workspace/pull/899)) Fix `Started` event not emitted to JS when downloading update.
- [`8505a75`](https://github.com/tauri-apps/plugins-workspace/commit/8505a756b569d88757ec58e452bfe4814d8107bf)([#907](https://github.com/tauri-apps/plugins-workspace/pull/907)) Add support for specifying proxy to use for checking and downloading updates.

## \[2.0.0-alpha.5]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.13.
- [`e5f979f`](https://github.com/tauri-apps/plugins-workspace/commit/e5f979f91abbb1775fa048af3219b30ff30ed691)([#818](https://github.com/tauri-apps/plugins-workspace/pull/818)) Fix NSIS updater failing to launch when using `basicUi` mode.

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
- [`4ab90f0`](https://github.com/tauri-apps/plugins-workspace/commit/4ab90f048eab2918344f97dc8e04413a404e392d)([#431](https://github.com/tauri-apps/plugins-workspace/pull/431)) The updater plugin is recieving a few changes to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  ater.
- [`1cb8311`](https://github.com/tauri-apps/plugins-workspace/commit/1cb831183c63ba5bd3f72d8a482992f6467d950d)([#405](https://github.com/tauri-apps/plugins-workspace/pull/405)) Implement passive mode on NSIS and automatically restart after NSIS update.
- [`4ab90f0`](https://github.com/tauri-apps/plugins-workspace/commit/4ab90f048eab2918344f97dc8e04413a404e392d)([#431](https://github.com/tauri-apps/plugins-workspace/pull/431)) The updater plugin is recieving a few changes to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  0f048eab2918344f97dc8e04413a404e392d)([#431](https://github.com/tauri-apps/plugins-workspace/pull/431)) The updater plugin is recieving a few changes to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  1]\(https://github.com/tauri-apps/plugins-workspace/pull/431)) The updater plugin is recieving a few changes to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  0f048eab2918344f97dc8e04413a404e392d)([#431](https://github.com/tauri-apps/plugins-workspace/pull/431)) The updater plugin is recieving a few changes to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  918344f97dc8e04413a404e392d)([#431](https://github.com/tauri-apps/plugins-workspace/pull/431)) The updater plugin is recieving a few changes to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  0f048eab2918344f97dc8e04413a404e392d)([#431](https://github.com/tauri-apps/plugins-workspace/pull/431)) The updater plugin is recieving a few changes to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  1]\(https://github.com/tauri-apps/plugins-workspace/pull/431)) The updater plugin is recieving a few changes to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  0f048eab2918344f97dc8e04413a404e392d)([#431](https://github.com/tauri-apps/plugins-workspace/pull/431)) The updater plugin is recieving a few changes to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  92fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  i-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  92fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  to improve consistency and ergonomics of the Rust and JS APIs

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  92fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  i-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  92fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
