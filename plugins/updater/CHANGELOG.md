# Changelog

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
