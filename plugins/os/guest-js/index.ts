// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Provides operating system-related utility methods and properties.
 *
 * @module
 */

import { invoke } from '@tauri-apps/api/core'

/** @ignore */
declare global {
  interface Window {
    __TAURI_OS_PLUGIN_INTERNALS__: {
      eol: string
      os_type: OsType
      platform: Platform
      family: Family
      version: string
      arch: Arch
      exe_extension: string
    }
  }
}

/**
 * A string describing the specific operating system in use, as returned by {@link platform}.
 */
type Platform =
  | 'linux'
  | 'macos'
  | 'ios'
  | 'freebsd'
  | 'dragonfly'
  | 'netbsd'
  | 'openbsd'
  | 'solaris'
  | 'android'
  | 'windows'

/**
 * A coarse-grained operating system category, as returned by {@link type}.
 */
type OsType = 'linux' | 'windows' | 'macos' | 'ios' | 'android'

/**
 * A string describing the specific operating system architecture in use, as returned by {@link arch}.
 */
type Arch =
  | 'x86'
  | 'x86_64'
  | 'arm'
  | 'aarch64'
  | 'mips'
  | 'mips64'
  | 'powerpc'
  | 'powerpc64'
  | 'riscv64'
  | 's390x'
  | 'sparc64'

/**
 * Returns the operating system-specific end-of-line marker.
 * - `\n` on POSIX
 * - `\r\n` on Windows
 *
 * @example
 * ```typescript
 * import { eol } from '@tauri-apps/plugin-os';
 * const eolChar = eol();
 * ```
 *
 * @returns The end-of-line marker for the current platform.
 * @since 2.0.0
 * */
function eol(): string {
  return window.__TAURI_OS_PLUGIN_INTERNALS__.eol
}

/**
 * Returns a string describing the specific operating system in use.
 * The value is set at compile time. Possible values are `'linux'`, `'macos'`, `'ios'`, `'freebsd'`, `'dragonfly'`, `'netbsd'`, `'openbsd'`, `'solaris'`, `'android'`, `'windows'`
 *
 * @example
 * ```typescript
 * import { platform } from '@tauri-apps/plugin-os';
 * const platformName = platform();
 * ```
 *
 * @returns The platform name.
 * @since 2.0.0
 *
 */
function platform(): Platform {
  return window.__TAURI_OS_PLUGIN_INTERNALS__.platform
}

/**
 * Returns the current operating system version.
 *
 * On Android this is the user-visible release (e.g. `14.0.0`) and on iOS the system version (e.g. `17.4.1`).
 * Before 2.5.0 it was `Unknown` on both.
 *
 * @example
 * ```typescript
 * import { version } from '@tauri-apps/plugin-os';
 * const osVersion = version();
 * ```
 *
 * @returns The operating system version.
 * @since 2.0.0
 */
function version(): string {
  return window.__TAURI_OS_PLUGIN_INTERNALS__.version
}

/**
 * A string describing the operating system family, as returned by {@link family}.
 */
type Family = 'unix' | 'windows'

/**
 * Returns the current operating system family. Possible values are `'unix'`, `'windows'`.
 * @example
 * ```typescript
 * import { family } from '@tauri-apps/plugin-os';
 * const family = family();
 * ```
 *
 * @returns The operating system family.
 * @since 2.0.0
 */
function family(): Family {
  return window.__TAURI_OS_PLUGIN_INTERNALS__.family
}

/**
 * Returns the current operating system type. Returns `'linux'` on Linux, `'macos'` on macOS, `'windows'` on Windows, `'ios'` on iOS and `'android'` on Android.
 * @example
 * ```typescript
 * import { type } from '@tauri-apps/plugin-os';
 * const osType = type();
 * ```
 *
 * @returns The operating system type.
 * @since 2.0.0
 */
function type(): OsType {
  return window.__TAURI_OS_PLUGIN_INTERNALS__.os_type
}

/**
 * Returns the current operating system architecture.
 * Possible values are `'x86'`, `'x86_64'`, `'arm'`, `'aarch64'`, `'mips'`, `'mips64'`, `'powerpc'`, `'powerpc64'`, `'riscv64'`, `'s390x'`, `'sparc64'`.
 * @example
 * ```typescript
 * import { arch } from '@tauri-apps/plugin-os';
 * const archName = arch();
 * ```
 *
 * @returns The operating system architecture.
 * @since 2.0.0
 */
function arch(): Arch {
  return window.__TAURI_OS_PLUGIN_INTERNALS__.arch
}

/**
 * Returns the file extension, if any, used for executable binaries on this platform. Possible values are `'exe'` and `''` (empty string).
 * @example
 * ```typescript
 * import { exeExtension } from '@tauri-apps/plugin-os';
 * const exeExt = exeExtension();
 * ```
 *
 * @returns The file extension used for executable binaries on this platform.
 * @since 2.0.0
 */
function exeExtension(): string {
  return window.__TAURI_OS_PLUGIN_INTERNALS__.exe_extension
}

/**
 * Returns a String with a `BCP-47` language tag inside. If the locale couldn’t be obtained, `null` is returned instead.
 * @example
 * ```typescript
 * import { locale } from '@tauri-apps/plugin-os';
 * const locale = await locale();
 * if (locale) {
 *    // use the locale string here
 * }
 * ```
 *
 * @returns A promise resolving to the `BCP-47` language tag, or `null` if it could not be obtained.
 * @since 2.0.0
 */
async function locale(): Promise<string | null> {
  return await invoke('plugin:os|locale')
}

/**
 * Returns the host name of the operating system.
 * @example
 * ```typescript
 * import { hostname } from '@tauri-apps/plugin-os';
 * const hostname = await hostname();
 * ```
 *
 * @returns A promise resolving to the host name of the operating system.
 * @since 2.0.0
 */
async function hostname(): Promise<string | null> {
  return await invoke('plugin:os|hostname')
}

/**
 * Information about the device, as returned by {@link deviceInfo}.
 *
 * Every field is `null` when the platform cannot provide it.
 *
 * @since 2.5.0
 */
interface DeviceInfo {
  /**
   * The device model identifier.
   *
   * - **Android:** [`Build.MODEL`](https://developer.android.com/reference/android/os/Build#MODEL), e.g. `Pixel 8`.
   * - **iOS:** the machine identifier, e.g. `iPhone15,2`. On the simulator, the identifier of the simulated device.
   * - **macOS:** the `hw.model` sysctl, e.g. `Mac14,2`.
   * - **Linux:** the DMI product name, or the device tree model on boards without DMI (e.g. a Raspberry Pi).
   * - **Windows:** the `SystemProductName` reported by the firmware.
   *
   * Desktop values come straight from the firmware and can be placeholders such as `To Be Filled By O.E.M.`.
   */
  model: string | null
  /**
   * The device manufacturer.
   *
   * - **Android:** [`Build.MANUFACTURER`](https://developer.android.com/reference/android/os/Build#MANUFACTURER), e.g. `Google`.
   * - **iOS** and **macOS:** always `Apple`.
   * - **Linux:** the DMI system vendor.
   * - **Windows:** the `SystemManufacturer` reported by the firmware.
   */
  manufacturer: string | null
  /**
   * Whether the app is running on a simulator or emulator.
   *
   * - **Android:** a heuristic based on the build fingerprint and hardware name, which recognizes the
   *   Android SDK emulator but not every third-party one.
   * - **iOS:** whether the app was built for the simulator.
   * - **Desktop:** always `null`. Virtual machines are not detected.
   */
  isVirtual: boolean | null
  /**
   * The total physical memory of the device in bytes.
   *
   * On Android this is the memory available to the kernel, which is slightly less than the installed RAM.
   */
  totalMemory: number | null
  /**
   * The version of the system webview.
   *
   * - **Android:** the version of the WebView package, e.g. `120.0.6099.230`. `null` on Android 7 (API 24 and 25).
   * - **iOS:** the `CFBundleVersion` of the WebKit framework.
   * - **Desktop:** always `null`.
   */
  webviewVersion: string | null
}

/**
 * Battery information, as returned by {@link batteryInfo}.
 *
 * @since 2.5.0
 */
interface BatteryInfo {
  /** The charge level, from `0` (empty) to `1` (full). `null` if it is unknown, such as on the iOS simulator. */
  level: number | null
  /** Whether the device is plugged in and charging (or full). `null` if it is unknown, such as on the iOS simulator. */
  isCharging: boolean | null
}

/**
 * Returns information about the device: its model, manufacturer, whether it is a simulator or emulator,
 * its total memory and the system webview version. See {@link DeviceInfo} for what each platform reports.
 *
 * @example
 * ```typescript
 * import { deviceInfo } from '@tauri-apps/plugin-os';
 * const { model, manufacturer, isVirtual } = await deviceInfo();
 * ```
 *
 * @returns A promise resolving to the device information.
 * @since 2.5.0
 */
async function deviceInfo(): Promise<DeviceInfo> {
  return await invoke('plugin:os|device_info')
}

/**
 * Returns the name the user gave the device, e.g. `My Pixel`.
 *
 * #### Platform-specific
 *
 * - **Android:** the device name from the system settings (Android 7.1+). `null` on Android 7.0 or if the user never set one.
 * - **iOS:** since iOS 16 this is a generic name such as `iPhone` unless the app has the
 *   [`com.apple.developer.device-information.user-assigned-device-name`](https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.developer.device-information.user-assigned-device-name)
 *   entitlement, which Apple only grants on request.
 * - **Desktop:** always `null`. Use {@link hostname} instead.
 *
 * @example
 * ```typescript
 * import { deviceName } from '@tauri-apps/plugin-os';
 * const name = await deviceName();
 * ```
 *
 * @returns A promise resolving to the device name, or `null` if it is not available.
 * @since 2.5.0
 */
async function deviceName(): Promise<string | null> {
  return await invoke('plugin:os|device_name')
}

/**
 * Returns the battery level and charging state.
 *
 * #### Platform-specific
 *
 * - **Android / iOS:** always an object, but its fields are `null` when the state is unknown (e.g. on the iOS simulator).
 * - **Desktop:** always `null`.
 *
 * @example
 * ```typescript
 * import { batteryInfo } from '@tauri-apps/plugin-os';
 * const battery = await batteryInfo();
 * if (battery?.level != null) {
 *   console.log(`battery at ${Math.round(battery.level * 100)}%`);
 * }
 * ```
 *
 * @returns A promise resolving to the battery information, or `null` on desktop.
 * @since 2.5.0
 */
async function batteryInfo(): Promise<BatteryInfo | null> {
  return await invoke('plugin:os|battery_info')
}

export {
  eol,
  platform,
  family,
  version,
  type,
  arch,
  locale,
  exeExtension,
  hostname,
  deviceInfo,
  deviceName,
  batteryInfo
}
export type { Platform, OsType, Arch, Family, DeviceInfo, BatteryInfo }
