# Changelog

## \[2.2.0]

- [`3a79266b`](https://github.com/tauri-apps/plugins-workspace/commit/3a79266b8cf96a55b1ae6339d725567d45a44b1d) ([#2173](https://github.com/tauri-apps/plugins-workspace/pull/2173) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Bumped all plugins to `v2.2.0`. From now, the versions for the Rust and JavaScript packages of each plugin will be in sync with each other.

## \[2.1.0]

- [`4db62635`](https://github.com/tauri-apps/plugins-workspace/commit/4db626354c8e29e37bedcf94787a8dd36ce21c55) ([#2076](https://github.com/tauri-apps/plugins-workspace/pull/2076) by [@jakobwesthoff](https://github.com/tauri-apps/plugins-workspace/../../jakobwesthoff)) Add `moveWindowConstrained` function that is similar to `moveWindow` but constrains the window to the screen dimensions in case of tray icon positions.

## \[2.0.1]

- [`3c1f3874`](https://github.com/tauri-apps/plugins-workspace/commit/3c1f3874f4c828637b3aa983cba13c77427faf58) ([#1911](https://github.com/tauri-apps/plugins-workspace/pull/1911) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Added missing permission for `handleIconState` and fixed its event processing logic.

## \[2.0.1]

- [`a1a82208`](https://github.com/tauri-apps/plugins-workspace/commit/a1a82208ed4ab87f83310be0dc95428aec9ab241) ([#1873](https://github.com/tauri-apps/plugins-workspace/pull/1873) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

## \[2.0.0]

- [`e2c4dfb6`](https://github.com/tauri-apps/plugins-workspace/commit/e2c4dfb6af43e5dd8d9ceba232c315f5febd55c1) Update to tauri v2 stable release.

## \[2.0.0-rc.2]

- [`2f7e32b5`](https://github.com/tauri-apps/plugins-workspace/commit/2f7e32b5e07454d6c0cf3ab03f8af8da74c4a8a7) ([#1822](https://github.com/tauri-apps/plugins-workspace/pull/1822) by [@jbolda](https://github.com/tauri-apps/plugins-workspace/../../jbolda)) `handleIconState` function for use in JavaScript event handlers. This allows one to update the TrayIcon state through JavaScript and fully create and handle the TrayIcon without requiring Rust (and the side-effect of creating a TrayIcon).

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

## \[2.0.0-beta.5]

- [`d9de5b19`](https://github.com/tauri-apps/plugins-workspace/commit/d9de5b19d1e950c06f0915ae92a862acb266d108)([#1283](https://github.com/tauri-apps/plugins-workspace/pull/1283)) Implement `WindowExt` for `WebviewWindow`.

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

## \[1.0.5]

- `TrayLeft`, `TrayRight` and `TrayCenter` will now position the window according to the tray position relative to the monitor dimensions to prevent windows being displayed partially off-screen.
  - [3d27909](https://github.com/tauri-apps/plugins-workspace/commit/3d279094d44be78cdc5d1de3938f1414e13db6b0) fix(positioner): Prevent tray relative windows from being moved off-screen ([#291](https://github.com/tauri-apps/plugins-workspace/pull/291)) on 2023-09-27

## \[0.2.7]

- Update Tauri to v1.0.0
  - Bumped due to a bump in tauri-plugin-positioner.
  - [0bb73eb](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/0bb73eb20dae87f730c0b5f4cc08e6689e25fdba) Create tauri-v1.md on 2022-06-16

## \[0.2.6]

- Update Tauri to v1.0.0-rc.12
  - Bumped due to a bump in tauri-plugin-positioner.
  - [de6d76f](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/de6d76f3a96a68e88a7ac434d65df4dbc82af122) Create update-tauri.md on 2022-05-25

## \[0.2.5]

- Update deps
  - Bumped due to a bump in tauri-plugin-positioner.
  - [a8d3f73](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/a8d3f73b74829ef5d53d4fb028e59d09e8946399) Create chore-update-deps.md on 2022-05-18

## \[0.2.4]

- Update Tauri dependencies
  - [2095b6a](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/2095b6a4a4ab5590add099ddb2b1e8118e3496e4) add dep update changefile on 2022-02-14
  - [53d3a50](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/53d3a501776f16741124aa961f521b9d7798c878) publish new versions ([#42](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/pull/42)) on 2022-02-14
  - [9f32726](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/9f32726ede38bb9b2711f738a2f9fee7f0da2d73) Create update-deps.md on 2022-05-11

## \[0.2.3]

- **Breaking Change**: Uses the new Tauri plugin builder pattern. Use `tauri_plugin_positioner::init()` instead of `tauri_plugin_positioner::Positioner::default()`.
  - Bumped due to a bump in tauri-plugin-positioner.
  - [14837a8](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/14837a8d9cecdd6014867d4ef00fb98f21b2249d) refactor: use new builder pattern on 2022-02-26
  - [59874d8](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/59874d827471dfb889662fadc74fec1f2243b89e) fix typo on 2022-02-26

## \[0.2.2]

- Update README.md
  - Bumped due to a bump in tauri-plugin-positioner.
  - [92d6c3d](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/92d6c3dca00a6b3562682804a649c0023831ce2b) Create docs-update-readme.md on 2022-02-17

## \[0.2.1]

- Update `tauri` to `1.0.0-rc.1`, `serde` to `1.0.136` and `serde_json` to `1.0.79`.
  - [2095b6a](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/2095b6a4a4ab5590add099ddb2b1e8118e3496e4) add dep update changefile on 2022-02-14

## \[0.2.0]

- Add SystemTray relative positions.
  - [765b3ed](https://www.github.com/JonasKruckenberg/tauri-plugin-positioner/commit/765b3ed90056d85ae88b0852b7107ff2b84a6c3a) Create feat-tray-pos.md on 2022-01-19

## \[0.1.0]

- Update package/crate metadata
