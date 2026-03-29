# Changelog

## \[2.8.0]

- [`2a625adf`](https://github.com/tauri-apps/plugins-workspace/commit/2a625adff30238904035b86b6e2db7595597e857) ([#3065](https://github.com/tauri-apps/plugins-workspace/pull/3065) by [@BinaryMuse](https://github.com/tauri-apps/plugins-workspace/../../BinaryMuse)) Allow specifying a log formatter per target using the `format` method on `Target`.
- [`ae278ddf`](https://github.com/tauri-apps/plugins-workspace/commit/ae278ddf60203da183fe2266c06a5bdeb909285c) ([#3110](https://github.com/tauri-apps/plugins-workspace/pull/3110) by [@liuzhch1](https://github.com/tauri-apps/plugins-workspace/../../liuzhch1)) Fix log file rotation when exceeding `max_file_size`.
- [`6de61f85`](https://github.com/tauri-apps/plugins-workspace/commit/6de61f854bee0c1d39e4ce5890b12754b931c163) ([#3113](https://github.com/tauri-apps/plugins-workspace/pull/3113) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Remove log delays for iOS simulators that are no longer necessary.

## \[2.7.1]

- [`93426f85`](https://github.com/tauri-apps/plugins-workspace/commit/93426f85120f49beb9f40222bff45185a32d54a9) Fixed an issue that caused docs.rs builds to fail. No user facing changes.

## \[2.7.0]

- [`625bb1c0`](https://github.com/tauri-apps/plugins-workspace/commit/625bb1c0965394b88522643731f78ccbcca84add) ([#2965](https://github.com/tauri-apps/plugins-workspace/pull/2965) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Re-export the log crate.

## \[2.6.0]

- [`f209b2f2`](https://github.com/tauri-apps/plugins-workspace/commit/f209b2f23cb29133c97ad5961fb46ef794dbe063) ([#2804](https://github.com/tauri-apps/plugins-workspace/pull/2804) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) Updated tauri to 2.6

## \[2.5.1]

### bug

- [`9799f0db`](https://github.com/tauri-apps/plugins-workspace/commit/9799f0dbabea0b572a9b9111954fbf9aca63da71) ([#2802](https://github.com/tauri-apps/plugins-workspace/pull/2802) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Fixes iOS simulator still freezing sometimes due to early logging.

## \[2.5.0]

- [`106e46ed`](https://github.com/tauri-apps/plugins-workspace/commit/106e46ed5125be33d0427cab9c5c066802f68791) ([#677](https://github.com/tauri-apps/plugins-workspace/pull/677)) Added the `KeepSome` rotation strategy. Like `KeepAll` it will rename files when the max file size is exceeded but will keep only the specified amount of files around.

## \[2.4.0]

- [`c9b21f6f`](https://github.com/tauri-apps/plugins-workspace/commit/c9b21f6f4345806eff5f495885f20dea0082b7d7) ([#2625](https://github.com/tauri-apps/plugins-workspace/pull/2625) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Export the `LogLevel` type.
- [`9629c2f4`](https://github.com/tauri-apps/plugins-workspace/commit/9629c2f4f90a56b5c2d265d1d13d3af40fc0c525) ([#2600](https://github.com/tauri-apps/plugins-workspace/pull/2600) by [@exoego](https://github.com/tauri-apps/plugins-workspace/../../exoego)) Adds a new varient `TargetKind::Dispatch` that allows you to construct arbitrary log targets
- [`686a839c`](https://github.com/tauri-apps/plugins-workspace/commit/686a839c96fae1b0334f2df9dc76ca5cdbe00dbe) ([#2626](https://github.com/tauri-apps/plugins-workspace/pull/2626) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Fix iOS app stuck when using the iOS Simulator and the log plugin due to a deadlock when calling os_log too early.

### feat

- [`60fc35d3`](https://github.com/tauri-apps/plugins-workspace/commit/60fc35d35cccaf1654eceb4446ecf0f89dc15502) ([#2576](https://github.com/tauri-apps/plugins-workspace/pull/2576) by [@3lpsy](https://github.com/tauri-apps/plugins-workspace/../../3lpsy)) Add a `tracing` feature to the `log` plugin that emits log messages to the `tracing` system.

## \[2.3.1]

- [`1bb1ced5`](https://github.com/tauri-apps/plugins-workspace/commit/1bb1ced53820127204aa7adf57510c1cbce55e12) ([#2524](https://github.com/tauri-apps/plugins-workspace/pull/2524) by [@elwerene](https://github.com/tauri-apps/plugins-workspace/../../elwerene)) enable TargetKind::LogDir on mobile

## \[2.3.0]

### feat

- [`02481501`](https://github.com/tauri-apps/plugins-workspace/commit/024815018fbc63a37afc716796a454925aa7d25e) ([#2377](https://github.com/tauri-apps/plugins-workspace/pull/2377) by [@3lpsy](https://github.com/tauri-apps/plugins-workspace/../../3lpsy)) Add a `is_skip_logger` flag to the Log Plugin `Builder` struct, a `skip_logger()` method to the Builder, and logic to avoid acquiring (creating) a logger and attaching it to the global logger. Since acquire_logger is pub, a `LoggerNotInitialized` is added and returned if it's called when the `is_skip_looger` flag is set. Overall, this feature permits a user to avoid calling `attach_logger` which can only be called once in a program's lifetime and allows the user to control the logger returned from `logger()`. Additionally, it also will allow users to generate multiple Tauri Mock apps in test suites that run and parallel and have the `log` plugin attached (assuming they use `skip_logger()`).

## \[2.2.3]

- [`1a984659`](https://github.com/tauri-apps/plugins-workspace/commit/1a9846599b6a71faf330845847a30f6bf9735898) ([#2469](https://github.com/tauri-apps/plugins-workspace/pull/2469) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Update `objc2` crate to 0.6. No user facing changes.

## \[2.2.2]

- [`6b4c3917`](https://github.com/tauri-apps/plugins-workspace/commit/6b4c3917389f4bc489d03b48a837557ac0584175) ([#2401](https://github.com/tauri-apps/plugins-workspace/pull/2401) by [@Seishiin](https://github.com/tauri-apps/plugins-workspace/../../Seishiin)) Fix timezone_strategy overwriting previously set LogLevels.

## \[2.2.1]

- [`784a54a3`](https://github.com/tauri-apps/plugins-workspace/commit/784a54a39094dfbaaa8dd123eb083c04dc6c3bb2) ([#2344](https://github.com/tauri-apps/plugins-workspace/pull/2344) by [@madsmtm](https://github.com/tauri-apps/plugins-workspace/../../madsmtm)) Use `objc2` instead of `objc`.

## \[2.2.0]

- [`3a79266b`](https://github.com/tauri-apps/plugins-workspace/commit/3a79266b8cf96a55b1ae6339d725567d45a44b1d) ([#2173](https://github.com/tauri-apps/plugins-workspace/pull/2173) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) Bumped all plugins to `v2.2.0`. From now, the versions for the Rust and JavaScript packages of each plugin will be in sync with each other.

## \[2.0.2]

- [`69d508ee`](https://github.com/tauri-apps/plugins-workspace/commit/69d508ee6910ae4064f2398fbacb803b3944d6a8) ([#2157](https://github.com/tauri-apps/plugins-workspace/pull/2157) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Make log functions omit caller location when failed to parse it instead of throwing

## \[2.0.1]

- [`371a2f73`](https://github.com/tauri-apps/plugins-workspace/commit/371a2f7361e0b91cf66f1287ffb18b34414a6cb8) ([#2021](https://github.com/tauri-apps/plugins-workspace/pull/2021) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Make webview log target more consistent that it always starts with `webview`

## \[2.0.2]

- [`606fa08d`](https://github.com/tauri-apps/plugins-workspace/commit/606fa08dae1acd074b961fb360623f4c86f13ee8) ([#1997](https://github.com/tauri-apps/plugins-workspace/pull/1997) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) **Potentially breaking:** Updated `fern` from 0.6 to 0.7. This is technically a breaking change because `fern` is re-exported in `tauri-plugin-log`.

## \[2.0.1]

- [`a1a82208`](https://github.com/tauri-apps/plugins-workspace/commit/a1a82208ed4ab87f83310be0dc95428aec9ab241) ([#1873](https://github.com/tauri-apps/plugins-workspace/pull/1873) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Downgrade MSRV to 1.77.2 to support Windows 7.

## \[2.0.0]

- [`e2c4dfb6`](https://github.com/tauri-apps/plugins-workspace/commit/e2c4dfb6af43e5dd8d9ceba232c315f5febd55c1) Update to tauri v2 stable release.

## \[2.0.0-rc.1]

- [`e2e97db5`](https://github.com/tauri-apps/plugins-workspace/commit/e2e97db51983267f5be84d4f6f0278d58834d1f5) ([#1701](https://github.com/tauri-apps/plugins-workspace/pull/1701) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri 2.0.0-rc.8

## \[2.0.0-rc.1]

- [`b9147758`](https://github.com/tauri-apps/plugins-workspace/commit/b914775898c2bee7ceb20bd17ee595005cd17a64) ([#1679](https://github.com/tauri-apps/plugins-workspace/pull/1679) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Explicitly set a minimum macOS version for the Swift package.

## \[2.0.0-rc.0]

- [`9887d1`](https://github.com/tauri-apps/plugins-workspace/commit/9887d14bd0e971c4c0f5c1188fc4005d3fc2e29e) Update to tauri RC.

## \[2.0.0-beta.9]

- [`99d6ac0f`](https://github.com/tauri-apps/plugins-workspace/commit/99d6ac0f9506a6a4a1aa59c728157190a7441af6) ([#1606](https://github.com/tauri-apps/plugins-workspace/pull/1606) by [@FabianLars](https://github.com/tauri-apps/plugins-workspace/../../FabianLars)) The JS packages now specify the *minimum* `@tauri-apps/api` version instead of a single exact version.
- [`6de87966`](https://github.com/tauri-apps/plugins-workspace/commit/6de87966ecc00ad9d91c25be452f1f46bd2b7e1f) ([#1597](https://github.com/tauri-apps/plugins-workspace/pull/1597) by [@Legend-Master](https://github.com/tauri-apps/plugins-workspace/../../Legend-Master)) Update to tauri beta.25.

## \[2.0.0-beta.9]

- [`20a1d24e`](https://github.com/tauri-apps/plugins-workspace/commit/20a1d24ee004e77c2d12a0e20d258ce120216ed1) ([#1579](https://github.com/tauri-apps/plugins-workspace/pull/1579) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Added `Builder::split` which returns the raw logger implementation so you can pipe to other loggers such as `multi_log` or `tauri-plugin-devtools`.

## \[2.0.0-beta.8]

- [`22a17980`](https://github.com/tauri-apps/plugins-workspace/commit/22a17980ff4f6f8c40adb1b8f4ffc6dae2fe7e30) ([#1537](https://github.com/tauri-apps/plugins-workspace/pull/1537) by [@lucasfernog](https://github.com/tauri-apps/plugins-workspace/../../lucasfernog)) Update to tauri beta.24.

## \[2.0.0-beta.7]

- [`76daee7a`](https://github.com/tauri-apps/plugins-workspace/commit/76daee7aafece34de3092c86e531cf9eb1138989) ([#1512](https://github.com/tauri-apps/plugins-workspace/pull/1512) by [@renovate](https://github.com/tauri-apps/plugins-workspace/../../renovate)) Update to tauri beta.23.

## \[2.0.0-beta.6]

- [`9013854f`](https://github.com/tauri-apps/plugins-workspace/commit/9013854f42a49a230b9dbb9d02774765528a923f)([#1382](https://github.com/tauri-apps/plugins-workspace/pull/1382)) Update to tauri beta.22.

## \[2.0.0-beta.5]

- [`430bd6f4`](https://github.com/tauri-apps/plugins-workspace/commit/430bd6f4f379bee5d232ae6b098ae131db7f178a)([#1363](https://github.com/tauri-apps/plugins-workspace/pull/1363)) Update to tauri beta.20.

## \[2.0.0-beta.4]

- [`bd1ed590`](https://github.com/tauri-apps/plugins-workspace/commit/bd1ed5903ffcce5500310dac1e59e8c67674ef1e)([#1237](https://github.com/tauri-apps/plugins-workspace/pull/1237)) Update to tauri beta.17.

## \[2.0.0-beta.3]

- [`ed46dca`](https://github.com/tauri-apps/plugins-workspace/commit/ed46dca74ff3947dbbcb26a7b571c129bf925698) Added `attachLogger` helper function to register a function that should be called for each log entry.

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
