# Changelog

## \[2.0.3]

- [`ed981027`](https://github.com/tauri-apps/plugins-workspace/commit/ed981027dd4fba7d0e2f836eb5db34d344388d73) ([#1962](https://github.com/tauri-apps/plugins-workspace/pull/1962) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Improve performance of `readTextFile` and `readTextFileLines` APIs
- [`3e78173d`](https://github.com/tauri-apps/plugins-workspace/commit/3e78173df9ce90aa3b19e1f36d1f8712c5020fb6) ([#2018](https://github.com/tauri-apps/plugins-workspace/pull/2018) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Fix `readDir` function failing to read directories that contain broken symlinks.
- [`5092ea5e`](https://github.com/tauri-apps/plugins-workspace/commit/5092ea5e89817c0550d09b0a4ad17bf1253b23df) ([#1964](https://github.com/tauri-apps/plugins-workspace/pull/1964) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Add support for using `ReadableStream<Unit8Array>` with `writeFile` API.

## \[2.0.2]

- [`77149dc4`](https://github.com/tauri-apps/plugins-workspace/commit/77149dc4320d26b413e4a6bbe82c654367c51b32) ([#1965](https://github.com/tauri-apps/plugins-workspace/pull/1965) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Fix `writeTextFile` converting UTF-8 characters (for example `äöü`) in the given path into replacement character (`�`)

## \[2.0.3]

- [`14cee64c`](https://github.com/tauri-apps/plugins-workspace/commit/14cee64c82a72655ae6a4ac0892736a2959dbda5) ([#1958](https://github.com/tauri-apps/plugins-workspace/pull/1958) by [@bWanShiTong](https://github.com/tauri-apps/plugins-workspace/../../bWanShiTong)) Fix compilation on targets with pointer width of `16` or `32`

## \[2.0.1]

- [`ae802456`](https://github.com/tauri-apps/plugins-workspace/commit/ae8024565f074f313084777c8b10d1b5e3bbe220) ([#1950](https://github.com/tauri-apps/plugins-workspace/pull/1950) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Improve performance of the `FileHandle.read` and `writeTextFile` APIs.

## \[2.0.1]

- [`a1a82208`](https://github.com/tauri-apps/plugins-workspace/commit/a1a82208ed4ab87f83310be0dc95428aec9ab241) ([#1873](https://github.com/tauri-apps/plugins-workspace/pull/1873) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

## \[2.0.0]

- [`e2c4dfb6`](https://github.com/tauri-apps/plugins-workspace/commit/e2c4dfb6af43e5dd8d9ceba232c315f5febd55c1) Update to tauri v2 stable release.

## \[2.0.0-rc.6]

- [`fc9b189e`](https://github.com/tauri-apps/plugins-workspace/commit/fc9b189e83a29bd750714ec6336133c6eabdfa20) ([#1837](https://github.com/tauri-apps/plugins-workspace/pull/1837) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Fix failing to deserialize capability file when using an OS specific path in the scope that is not available on the current OS.

## \[2.0.0-rc.5]

- [`cc03ccf5`](https://github.com/tauri-apps/plugins-workspace/commit/cc03ccf5e0e4be8bbf50bbdebe957c84be7f779b) ([#1774](https://github.com/tauri-apps/plugins-workspace/pull/1774)) Fix `scope-app`, `scope-app-recursive` and `scope-index` not properly enabling the application paths.

## \[2.0.0-rc.4]

- [`9291e4d2`](https://github.com/tauri-apps/plugins-workspace/commit/9291e4d2caa31c883c71e55f2193bd8754d72f03) ([#1640](https://github.com/tauri-apps/plugins-workspace/pull/1640) by [@SRutile](https://github.com/tauri-apps/plugins-workspace/../../SRutile)) Support any UTF-8 character in the writeFile API.

## \[2.0.0-rc.3]

- [`a2fe5551`](https://github.com/tauri-apps/plugins-workspace/commit/a2fe55512f908dd11c814ce021d164f01677572a) ([#1727](https://github.com/tauri-apps/plugins-workspace/pull/1727) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Add utility methods on `FilePath` and `SafeFilePath` enums which are:

  - `path`
  - `simplified`
  - `into_path`
- [`a2fe5551`](https://github.com/tauri-apps/plugins-workspace/commit/a2fe55512f908dd11c814ce021d164f01677572a) ([#1727](https://github.com/tauri-apps/plugins-workspace/pull/1727) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Implement `Serialize`, `Deserialize`, `From`, `TryFrom` and `FromStr` traits for `FilePath` and `SafeFilePath` enums.
- [`a2fe5551`](https://github.com/tauri-apps/plugins-workspace/commit/a2fe55512f908dd11c814ce021d164f01677572a) ([#1727](https://github.com/tauri-apps/plugins-workspace/pull/1727) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Mark `Error` enum as `#[non_exhuastive]`.
- [`a2fe5551`](https://github.com/tauri-apps/plugins-workspace/commit/a2fe55512f908dd11c814ce021d164f01677572a) ([#1727](https://github.com/tauri-apps/plugins-workspace/pull/1727) by [@amrbashir](https://github.com/tauri-apps/plugins-workspace/../../amrbashir)) Add `SafeFilePath` enum.

## \[2.0.0-rc.2]

- [`f7280c88`](https://github.com/tauri-apps/plugins-workspace/commit/f7280c88309cdf1f2330574fec31e26e01e9cdbd) ([#1710](https://github.com/tauri-apps/plugins-workspace/pull/1710) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Fix can't use Windows paths like `C:/Users/UserName/file.txt`

### bug

- [`51819c60`](https://github.com/tauri-apps/plugins-workspace/commit/51819c601f863cbfbd11809a1c4cf9df5a20b1e0) ([#1708](https://github.com/tauri-apps/plugins-workspace/pull/1708) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Fixes `writeFile` command implementation on Android.

## \[2.0.0-rc.2]

- [`e2e97db5`](https://github.com/tauri-apps/plugins-workspace/commit/e2e97db51983267f5be84d4f6f0278d58834d1f5) ([#1701](https://github.com/tauri-apps/plugins-workspace/pull/1701) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri 2.0.0-rc.8

## \[2.0.0-rc.1]

- [`5f689902`](https://github.com/tauri-apps/plugins-workspace/commit/5f68990297f2cefac4220649a469adb7fa94fe1b) ([#1645](https://github.com/tauri-apps/plugins-workspace/pull/1645) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update documentation.

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
  kspace/pull/371)) First v2 alpha release!
  s/plugins-workspace/pull/371)) First v2 alpha release!
  ac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  .com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  uri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  .com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  kspace/pull/371)) First v2 alpha release!
  s/plugins-workspace/pull/371)) First v2 alpha release!
  ac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  .com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
apps/plugins-workspace/pull/371)) First v2 alpha release!
  .com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  kspace/pull/371)) First v2 alpha release!
  s/plugins-workspace/pull/371)) First v2 alpha release!
  ac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  .com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
  717ae670978feb4492fac1f295998b93f2b9347f)([#371](https://github.com/tauri-apps/plugins-workspace/pull/371)) First v2 alpha release!
