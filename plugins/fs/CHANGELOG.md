# Changelog

## \[2.0.0-beta.6]

- [`76daee7a`](https://github.com/tauri-apps/plugins-workspace/commit/76daee7aafece34de3092c86e531cf9eb1138989) ([#1512](https://github.com/tauri-apps/plugins-workspace/pull/1512) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) Update to tauri beta.23.

## \[2.0.0-beta.5]

- [`9013854f`](https://github.com/tauri-apps/plugins-workspace/commit/9013854f42a49a230b9dbb9d02774765528a923f)([#1382](https://github.com/tauri-apps/plugins-workspace/pull/1382)) Update to tauri beta.22.

## \[2.0.0-beta.4]

- [`430bd6f4`](https://github.com/tauri-apps/plugins-workspace/commit/430bd6f4f379bee5d232ae6b098ae131db7f178a)([#1363](https://github.com/tauri-apps/plugins-workspace/pull/1363)) Update to tauri beta.20.

## \[2.0.0-beta.3]

- [`bd1ed590`](https://github.com/tauri-apps/plugins-workspace/commit/bd1ed5903ffcce5500310dac1e59e8c67674ef1e)([#1237](https://github.com/tauri-apps/plugins-workspace/pull/1237)) Update to tauri beta.17.

## \[2.0.0-beta.6]

- [`b115fd22`](https://github.com/tauri-apps/plugins-workspace/commit/b115fd22e0da073f5d758c13474ec2106cf78163)([#1221](https://github.com/tauri-apps/plugins-workspace/pull/1221)) Fixes an issue that caused the app to freeze when the `dialog`, `fs`, and `persisted-scope` plugins were used together.

## \[2.0.0-beta.5]

- [`bb51a41`](https://github.com/tauri-apps/plugins-workspace/commit/bb51a41d67ebf989e8aedf10c4b1a7f9514d1bdf)([#1168](https://github.com/tauri-apps/plugins-workspace/pull/1168)) **Breaking Change:** All apis that return paths to the frontend will now remove the `\\?\` UNC prefix on Windows.
- [`e3d41f4`](https://github.com/tauri-apps/plugins-workspace/commit/e3d41f4011bd3ea3ce281bb38bbe31d3709f8e0f)([#1191](https://github.com/tauri-apps/plugins-workspace/pull/1191)) Internally use the webview scoped resources table instead of the app one, so other webviews can't access other webviews resources.
- [`e3d41f4`](https://github.com/tauri-apps/plugins-workspace/commit/e3d41f4011bd3ea3ce281bb38bbe31d3709f8e0f)([#1191](https://github.com/tauri-apps/plugins-workspace/pull/1191)) Update for tauri 2.0.0-beta.15.

## \[2.0.0-beta.4]

- [`9c2fb93`](https://github.com/tauri-apps/plugins-workspace/commit/9c2fb9306ecd3936a2aef56b3c012899036db098) Enhance the scope type to also allow a plain string representing the path to allow or deny.
- [`772f2bc`](https://github.com/tauri-apps/plugins-workspace/commit/772f2bc3495a4f83f1c3e538cbac6d29cbd7d5ef)([#1136](https://github.com/tauri-apps/plugins-workspace/pull/1136)) Update for tauri 2.0.0-beta.14.

## \[2.0.0-beta.3]

- [`cb96aa0`](https://github.com/tauri-apps/plugins-workspace/commit/cb96aa06277f7b864952827ec9fb1e74c8a1f761)([#1082](https://github.com/tauri-apps/plugins-workspace/pull/1082)) Fixes `watch` and `watchImmediate` which previously ignored the `baseDir` parameter.
- [`a04ea2f`](https://github.com/tauri-apps/plugins-workspace/commit/a04ea2f38294d5a3987578283badc8eec87a7752)([#1071](https://github.com/tauri-apps/plugins-workspace/pull/1071)) The global API script is now only added to the binary when the `withGlobalTauri` config is true.

## \[2.0.0-beta.2]

- [`7358102`](https://github.com/tauri-apps/plugins-workspace/commit/735810237e21504a027a65a7b3c25fd7e594288a)([#1029](https://github.com/tauri-apps/plugins-workspace/pull/1029)) Fix infinite loop on cargo build operations
- [`99bea25`](https://github.com/tauri-apps/plugins-workspace/commit/99bea2559c2c0648c2519c50a18cd124dacef57b)([#1005](https://github.com/tauri-apps/plugins-workspace/pull/1005)) Update to tauri beta.8.

## \[2.0.0-beta.1]

- [`569defb`](https://github.com/tauri-apps/plugins-workspace/commit/569defbe9492e38938554bb7bdc1be9151456d21) Update to tauri beta.4.

## \[2.0.0-beta.0]

- [`d198c01`](https://github.com/tauri-apps/plugins-workspace/commit/d198c014863ee260cb0de88a14b7fc4356ef7474)([#862](https://github.com/tauri-apps/plugins-workspace/pull/862)) Update to tauri beta.
- [`ea8eadc`](https://github.com/tauri-apps/plugins-workspace/commit/ea8eadce85b2e3e8eb7eb1a779fc3aa6c1201fa3)([#865](https://github.com/tauri-apps/plugins-workspace/pull/865)) Fix incorrect `create` option default value for `writeFile` and `writeTextFile` which didn't match documentation.
- [`61edbbe`](https://github.com/tauri-apps/plugins-workspace/commit/61edbbec0acda4213ed8684f75a973e8be123a52)([#885](https://github.com/tauri-apps/plugins-workspace/pull/885)) Replace `notify-debouncer-mini` with `notify-debouncer-full`. [(plugins-workspace#885)](https://github.com/tauri-apps/plugins-workspace/pull/885)

## \[2.0.0-alpha.6]

- [`85f8419`](https://github.com/tauri-apps/plugins-workspace/commit/85f841968200316958d707db0c39bb115f762471)([#848](https://github.com/tauri-apps/plugins-workspace/pull/848)) Fix `DebouncedEvent` type to correctly represent the actual type.
- [`c601230`](https://github.com/tauri-apps/plugins-workspace/commit/c60123093ddf725af7228494182fed697ff8b021)([#847](https://github.com/tauri-apps/plugins-workspace/pull/847)) Add `createNew` option for `writeFile` and `writeTextFile` to create the file if doesn't exist and fail if it does.
- [`c601230`](https://github.com/tauri-apps/plugins-workspace/commit/c60123093ddf725af7228494182fed697ff8b021)([#847](https://github.com/tauri-apps/plugins-workspace/pull/847)) Truncate files when using `writeFile` and `writeTextFile` with `append: false`.
- [`2e2fc8d`](https://github.com/tauri-apps/plugins-workspace/commit/2e2fc8de69dd8d282b66ec81561d57d8af802dc5)([#857](https://github.com/tauri-apps/plugins-workspace/pull/857)) Fix `invalid args id for command unwatch` error when trying to unwatch a previously watched file or directory.
- [`c601230`](https://github.com/tauri-apps/plugins-workspace/commit/c60123093ddf725af7228494182fed697ff8b021)([#847](https://github.com/tauri-apps/plugins-workspace/pull/847)) Fix panic when using `writeFile` or `writeTextFile` without passing an option object.

## \[2.0.0-alpha.5]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.13.
- [`69a1fa0`](https://github.com/tauri-apps/plugins-workspace/commit/69a1fa099c3143b6e426492f1c9d9cfbe56d2209)([#751](https://github.com/tauri-apps/plugins-workspace/pull/751)) The `fs` plugin received a major overhaul to add new APIs and changed existing APIs to be closer to Node.js and Deno APIs.

## \[2.0.0-alpha.4]

- [`387c2f9`](https://github.com/tauri-apps/plugins-workspace/commit/387c2f9e0ce4c75c07ffa3fd76391a25b58f5daf)([#802](https://github.com/tauri-apps/plugins-workspace/pull/802)) Update to @tauri-apps/api v2.0.0-alpha.12.
- [`88d260d`](https://github.com/tauri-apps/plugins-workspace/commit/88d260d90130f9df4b9ce00c1ad1bf1e4b30b1c0)([#744](https://github.com/tauri-apps/plugins-workspace/pull/744)) Add second argument to `exists` function to specify base directory.

## \[2.0.0-alpha.3]

- [`e438e0a`](https://github.com/tauri-apps/plugins-workspace/commit/e438e0a62d4b430a5159f05f13ecd397dd891a0d)([#676](https://github.com/tauri-apps/plugins-workspace/pull/676)) Update to @tauri-apps/api v2.0.0-alpha.11.

## \[2.0.0-alpha.2]

- [`5c13736`](https://github.com/tauri-apps/plugins-workspace/commit/5c137365c60790e8d4037d449e8237aa3fffdab0)([#673](https://github.com/tauri-apps/plugins-workspace/pull/673)) Update to @tauri-apps/api v2.0.0-alpha.9.

## \[2.0.0-alpha.2]

- [`4e2cef9`](https://github.com/tauri-apps/plugins-workspace/commit/4e2cef9b702bbbb9cf4ee17de50791cb21f1b2a4)([#593](https://github.com/tauri-apps/plugins-workspace/pull/593)) Update to alpha.12.

## \[2.0.0-alpha.1]

- [`0bba693`](https://github.com/tauri-apps/plugins-workspace/commit/0bba6932c09da5267a9dbf75ba52252e39458420)([#454](https://github.com/tauri-apps/plugins-workspace/pull/454)) Fix `writeBinaryFile` crashing with `command 'write_binary_file' not found`
- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  /pull/454)) Fix `writeBinaryFile` crashing with `command 'write_binary_file' not found`
- [`d74fc0a`](https://github.com/tauri-apps/plugins-workspace/commit/d74fc0a097996e90a37be8f57d50b7d1f6ca616f)([#555](https://github.com/tauri-apps/plugins-workspace/pull/555)) Update to alpha.11.

## \[2.0.0-alpha.0]

- [`717ae67`](https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  ae67\`]\(https://github.com/tauri-apps/plugins-workspace/commit/717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  .com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  ac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  .com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
s/plugins-workspace/pull/371)) First v2 alpha release!
  ac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  .com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
