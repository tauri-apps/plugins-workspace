# Changelog

## \[2.3.0]

- [`78acfa45`](https://github.com/tauri-apps/plugins-workspace/commit/78acfa456f343c0af5fbf35660802d14ff7a5eca) ([#2027](https://github.com/tauri-apps/plugins-workspace/pull/2027) by [@twlite](https://github.com/tauri-apps/plugins-workspace/../../twlite)) `addListener` now returns a cleanup function instead of `void` to remove the listener.

## \[2.2.1]

- [`05cca602`](https://github.com/tauri-apps/plugins-workspace/commit/05cca602d927c30014d3892438ca28d99bc4e1d3) ([#2210](https://github.com/tauri-apps/plugins-workspace/pull/2210) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) **Breaking change:** Updated tokio_tungstenite to `0.26`. This may be a breaking change if you use the `tls_connector` Builder method in Rust.
  **Breaking change:** Removed the accidental `ConnectionConfig` struct re-export (in rust). This should not affect anyone since the Config was not used in any public API.

## \[2.2.0]

- [`3a79266b`](https://github.com/tauri-apps/plugins-workspace/commit/3a79266b8cf96a55b1ae6339d725567d45a44b1d) ([#2173](https://github.com/tauri-apps/plugins-workspace/pull/2173) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Bumped all plugins to `v2.2.0`. From now, the versions for the Rust and JavaScript packages of each plugin will be in sync with each other.

## \[2.0.1]

- [`a1a82208`](https://github.com/tauri-apps/plugins-workspace/commit/a1a82208ed4ab87f83310be0dc95428aec9ab241) ([#1873](https://github.com/tauri-apps/plugins-workspace/pull/1873) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

## \[2.0.0]

- [`e2c4dfb6`](https://github.com/tauri-apps/plugins-workspace/commit/e2c4dfb6af43e5dd8d9ceba232c315f5febd55c1) Update to tauri v2 stable release.

## \[2.0.0-rc.1]

- [`e2e97db5`](https://github.com/tauri-apps/plugins-workspace/commit/e2e97db51983267f5be84d4f6f0278d58834d1f5) ([#1701](https://github.com/tauri-apps/plugins-workspace/pull/1701) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri 2.0.0-rc.8

## \[2.0.0-rc.0]

- [`9887d1`](https://github.com/tauri-apps/plugins-workspace/commit/9887d14bd0e971c4c0f5c1188fc4005d3fc2e29e) Update to tauri RC.

## \[2.0.0-beta.8]

- [`99d6ac0f`](https://github.com/tauri-apps/plugins-workspace/commit/99d6ac0f9506a6a4a1aa59c728157190a7441af6) ([#1606](https://github.com/tauri-apps/plugins-workspace/pull/1606) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) The JS packages now specify the *minimum* `@tauri-apps/api` version instead of a single exact version.
- [`6de87966`](https://github.com/tauri-apps/plugins-workspace/commit/6de87966ecc00ad9d91c25be452f1f46bd2b7e1f) ([#1597](https://github.com/tauri-apps/plugins-workspace/pull/1597) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Update to tauri beta.25.

## \[2.0.0-beta.7]

- [`22a17980`](https://github.com/tauri-apps/plugins-workspace/commit/22a17980ff4f6f8c40adb1b8f4ffc6dae2fe7e30) ([#1537](https://github.com/tauri-apps/plugins-workspace/pull/1537) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri beta.24.

## \[2.0.0-beta.6]

- [`76daee7a`](https://github.com/tauri-apps/plugins-workspace/commit/76daee7aafece34de3092c86e531cf9eb1138989) ([#1512](https://github.com/tauri-apps/plugins-workspace/pull/1512) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) Update to tauri beta.23.

## \[2.0.0-beta.5]

- [`9013854f`](https://github.com/tauri-apps/plugins-workspace/commit/9013854f42a49a230b9dbb9d02774765528a923f)([#1382](https://github.com/tauri-apps/plugins-workspace/pull/1382)) Update to tauri beta.22.

## \[2.0.0-beta.4]

- [`430bd6f4`](https://github.com/tauri-apps/plugins-workspace/commit/430bd6f4f379bee5d232ae6b098ae131db7f178a)([#1363](https://github.com/tauri-apps/plugins-workspace/pull/1363)) Update to tauri beta.20.

## \[2.0.0-beta.3]

- [`bd1ed590`](https://github.com/tauri-apps/plugins-workspace/commit/bd1ed5903ffcce5500310dac1e59e8c67674ef1e)([#1237](https://github.com/tauri-apps/plugins-workspace/pull/1237)) Update to tauri beta.17.

## \[2.0.0-beta.4]

- [`ed46dca`](https://github.com/tauri-apps/plugins-workspace/commit/ed46dca74ff3947dbbcb26a7b571c129bf925698) **Breaking change:** Enable rustls by default and added a method to configure the TLS Connector for tungstenite.

## \[2.0.0-beta.3]

- [`a04ea2f`](https://github.com/tauri-apps/plugins-workspace/commit/a04ea2f38294d5a3987578283badc8eec87a7752)([#1071](https://github.com/tauri-apps/plugins-workspace/pull/1071)) The global API script is now only added to the binary when the `withGlobalTauri` config is true.

## \[2.0.0-beta.2]

- [`99bea25`](https://github.com/tauri-apps/plugins-workspace/commit/99bea2559c2c0648c2519c50a18cd124dacef57b)([#1005](https://github.com/tauri-apps/plugins-workspace/pull/1005)) Update to tauri beta.8.

## \[2.0.0-beta.1]

- [`569defb`](https://github.com/tauri-apps/plugins-workspace/commit/569defbe9492e38938554bb7bdc1be9151456d21) Update to tauri beta.4.

## \[2.0.0-beta.0]

- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Update to tauri beta.

## \[2.0.0-alpha.5]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.13.

## \[2.0.0-alpha.4]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.12.
- [`251852c`](https://github.com/tauri-apps/plugins-workspace/commit/251852ccbc97abe5765fb9663aab27701f3d7c7c)([#702](https://github.com/tauri-apps/plugins-workspace/pull/702)) Add support for custom request headers.

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
