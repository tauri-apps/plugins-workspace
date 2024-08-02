# Changelog

## \[2.0.0-rc.0]

- [`9887d1`](https://github.com/tauri-apps/plugins-workspace/commit/9887d14bd0e971c4c0f5c1188fc4005d3fc2e29e) Update to tauri RC.

## \[2.0.0-beta.9]

- [`99d6ac0f`](https://github.com/tauri-apps/plugins-workspace/commit/99d6ac0f9506a6a4a1aa59c728157190a7441af6) ([#1606](https://github.com/tauri-apps/plugins-workspace/pull/1606) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) The JS packages now specify the *minimum* `@tauri-apps/api` version instead of a single exact version.
- [`6de87966`](https://github.com/tauri-apps/plugins-workspace/commit/6de87966ecc00ad9d91c25be452f1f46bd2b7e1f) ([#1597](https://github.com/tauri-apps/plugins-workspace/pull/1597) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Update to tauri beta.25.

## \[2.0.0-beta.8]

- [`22a17980`](https://github.com/tauri-apps/plugins-workspace/commit/22a17980ff4f6f8c40adb1b8f4ffc6dae2fe7e30) ([#1537](https://github.com/tauri-apps/plugins-workspace/pull/1537) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri beta.24.

## \[2.0.0-beta.7]

- [`76daee7a`](https://github.com/tauri-apps/plugins-workspace/commit/76daee7aafece34de3092c86e531cf9eb1138989) ([#1512](https://github.com/tauri-apps/plugins-workspace/pull/1512) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) Update to tauri beta.23.

## \[2.0.0-beta.6]

- [`9013854f`](https://github.com/tauri-apps/plugins-workspace/commit/9013854f42a49a230b9dbb9d02774765528a923f)([#1382](https://github.com/tauri-apps/plugins-workspace/pull/1382)) Update to tauri beta.22.

## \[2.0.0-beta.5]

- [`430bd6f4`](https://github.com/tauri-apps/plugins-workspace/commit/430bd6f4f379bee5d232ae6b098ae131db7f178a)([#1363](https://github.com/tauri-apps/plugins-workspace/pull/1363)) Update to tauri beta.20.

## \[2.0.0-beta.7]

- [`d9de5b19`](https://github.com/tauri-apps/plugins-workspace/commit/d9de5b19d1e950c06f0915ae92a862acb266d108)([#1283](https://github.com/tauri-apps/plugins-workspace/pull/1283)) Implement `WindowExt` for `WebviewWindow`.

## \[2.0.0-beta.4]

- [`bd1ed590`](https://github.com/tauri-apps/plugins-workspace/commit/bd1ed5903ffcce5500310dac1e59e8c67674ef1e)([#1237](https://github.com/tauri-apps/plugins-workspace/pull/1237)) Update to tauri beta.17.

## \[2.0.0-beta.3]

- [`0e9541f`](https://github.com/tauri-apps/plugins-workspace/commit/0e9541fe8990395de7cc8887bc46b3f3665b44e1)([#1138](https://github.com/tauri-apps/plugins-workspace/pull/1138)) Add `Builder::with_filename` to support using a custom filename. Also add `AppHandleExt::file_name` and a similar function in JS, to retrieve it later.

## \[2.0.0-beta.4]

- [`c013fa5`](https://github.com/tauri-apps/plugins-workspace/commit/c013fa52cd66885cf457a64e75373cb2066bc849)([#1078](https://github.com/tauri-apps/plugins-workspace/pull/1078)) **Breaking change**: Changed the format of the state file from bincode to json. Also changed the filename to from `.window-state` to `.window-state.json`.

## \[2.0.0-beta.3]

- [`a04ea2f`](https://github.com/tauri-apps/plugins-workspace/commit/a04ea2f38294d5a3987578283badc8eec87a7752)([#1071](https://github.com/tauri-apps/plugins-workspace/pull/1071)) The global API script is now only added to the binary when the `withGlobalTauri` config is true.

## \[2.0.0-beta.2]

- [`99bea25`](https://github.com/tauri-apps/plugins-workspace/commit/99bea2559c2c0648c2519c50a18cd124dacef57b)([#1005](https://github.com/tauri-apps/plugins-workspace/pull/1005)) Update to tauri beta.8.

## \[2.0.0-beta.1]

- [`569defb`](https://github.com/tauri-apps/plugins-workspace/commit/569defbe9492e38938554bb7bdc1be9151456d21) Update to tauri beta.4.

## \[2.0.0-beta.0]

- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Update to tauri beta.

- [`14f59615`](https://github.com/tauri-apps/plugins-workspace/commit/14f5961569c7d759d8d6d836352c787484594bd5) Address a couple of issues with restoring positions:

  - Fix restoring window positions correctly when the top-left corner of the window was outside of the monitor.
  - Fix restore maximization state only maximized on main monitor.

## \[2.0.0-alpha.5]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.13.

## \[2.0.0-alpha.4]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.12.

## \[2.0.0-alpha.3]

- [`e438e0a`](https://github.com/tauri-apps/plugins-workspace/commit/e438e0a62d4b430a5159f05f13ecd397dd891a0d)([#676](https://github.com/tauri-apps/plugins-workspace/pull/676)) Update to @tauri-apps/api v2.0.0-alpha.11.

## \[2.0.0-alpha.2]

- [`5c13736`](https://github.com/tauri-apps/plugins-workspace/commit/5c137365c60790e8d4037d449e8237aa3fffdab0)([#673](https://github.com/tauri-apps/plugins-workspace/pull/673)) Update to @tauri-apps/api v2.0.0-alpha.9.
- [`beb6b13`](https://github.com/tauri-apps/plugins-workspace/commit/beb6b139eb669dc0346b3de919aed024f649b9d2)([#675](https://github.com/tauri-apps/plugins-workspace/pull/675)) Fix usage of no longer available `__TAURI_METADATA__` API.

## \[2.0.0-alpha.2]

- [`4e2cef9`](https://github.com/tauri-apps/plugins-workspace/commit/4e2cef9b702bbbb9cf4ee17de50791cb21f1b2a4)([#593](https://github.com/tauri-apps/plugins-workspace/pull/593)) Update to alpha.12.

## \[2.0.0-alpha.1]

- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.
- [`84b3612`](https://github.com/tauri-apps/plugins-workspace/commit/84b3612393e3d0d4faeebe1e61cb7d7973556503)([#436](https://github.com/tauri-apps/plugins-workspace/pull/436)) Correctly propagate the promise inside `saveWindowState`, `restoreState` and `restoreStateCurrent` so callers can choose to `await` them.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  lugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  /pull/371)) First v2 alpha release!
  lugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!

## \[0.1.1]

- Address a couple of issues with restoring positions:

- Fix restoring window positions correctly when the top-left corner of the window was outside of the monitor.

- Fix restore maximization state only maximized on main monitor.

- [70d9908](https://github.com/tauri-apps/plugins-workspace/commit/70d99086de3a58189d65c49954a3495972880725) fix(window-state): restore window position if the one of the window corners intersects with monitor ([#898](https://github.com/tauri-apps/plugins-workspace/pull/898)) on 2024-01-25
  sues with restoring positions:

- Fix restoring window positions correctly when the top-left corner of the window was outside of the monitor.

- Fix restore maximization state only maximized on main monitor.

- [70d9908](https://github.com/tauri-apps/plugins-workspace/commit/70d99086de3a58189d65c49954a3495972880725) fix(window-state): restore window position if the one of the window corners intersects with monitor ([#898](https://github.com/tauri-apps/plugins-workspace/pull/898)) on 2024-01-25
  ://github.com/tauri-apps/plugins-workspace/commit/70d99086de3a58189d65c49954a3495972880725) fix(window-state): restore window position if the one of the window corners intersects with monitor ([#898](https://github.com/tauri-apps/plugins-workspace/pull/898)) on 2024-01-25
  indow position if the one of the window corners intersects with monitor ([#898](https://github.com/tauri-apps/plugins-workspace/pull/898)) on 2024-01-25
  ://github.com/tauri-apps/plugins-workspace/commit/70d99086de3a58189d65c49954a3495972880725) fix(window-state): restore window position if the one of the window corners intersects with monitor ([#898](https://github.com/tauri-apps/plugins-workspace/pull/898)) on 2024-01-25
  ://github.com/tauri-apps/plugins-workspace/pull/898)) on 2024-01-25
