---
os: minor
os-js: minor
---

Added `deviceInfo`, `deviceName` and `batteryInfo` (`Os::device_info`, `Os::device_name` and `Os::battery_info` through the new `OsExt` trait in Rust) to read the device model, manufacturer, whether it is a simulator or emulator, its total memory, the system webview version, the user-assigned device name and the battery state. They are mainly meant for Android and iOS: on desktop only the model, manufacturer and total memory are reported, and `deviceName` and `batteryInfo` return `null`. `os:default` now includes `allow-device-info`; `allow-device-name` and `allow-battery-info` must be granted explicitly.

`version()` now reports the OS version on Android and iOS instead of `Unknown`.

The plugin now ships Android and iOS native code, so mobile apps compile its Kotlin library and Swift package.
