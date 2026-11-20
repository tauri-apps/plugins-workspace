// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation
import SwiftRs
import Tauri
import UIKit
import WebKit

// Optional fields that are nil are left out of the JSON and reach Rust as `None`.

struct DeviceInfo: Encodable {
  let model: String?
  let manufacturer: String
  let isVirtual: Bool
  let totalMemory: UInt64
  let webviewVersion: String?
}

struct DeviceName: Encodable {
  let name: String?
}

struct BatteryInfo: Encodable {
  let level: Float?
  let isCharging: Bool?
}

class OsPlugin: Plugin {
  @objc public func getDeviceInfo(_ invoke: Invoke) {
    invoke.resolve(
      DeviceInfo(
        model: modelIdentifier(),
        manufacturer: "Apple",
        isVirtual: isSimulator(),
        totalMemory: ProcessInfo.processInfo.physicalMemory,
        webviewVersion: webkitVersion()
      ))
  }

  @objc public func getDeviceName(_ invoke: Invoke) {
    // UIDevice is main-thread only.
    // Since iOS 16 this is a generic name (e.g. "iPhone") unless the app has the
    // com.apple.developer.device-information.user-assigned-device-name entitlement.
    DispatchQueue.main.async {
      invoke.resolve(DeviceName(name: UIDevice.current.name))
    }
  }

  @objc public func getBatteryInfo(_ invoke: Invoke) {
    DispatchQueue.main.async {
      let device = UIDevice.current
      // the battery level and state are only reported while monitoring is enabled;
      // restore the previous setting so the app's own monitoring is left untouched
      let wasMonitoring = device.isBatteryMonitoringEnabled
      device.isBatteryMonitoringEnabled = true
      let level = device.batteryLevel
      let state = device.batteryState
      device.isBatteryMonitoringEnabled = wasMonitoring

      invoke.resolve(
        BatteryInfo(
          // -1 when unknown, e.g. on the simulator
          level: level < 0 ? nil : level,
          isCharging: state == .unknown ? nil : (state == .charging || state == .full)
        ))
    }
  }

  /// The machine identifier, e.g. "iPhone15,2", rather than the generic UIDevice.model ("iPhone").
  private func modelIdentifier() -> String? {
    #if targetEnvironment(simulator)
      // uname reports the host Mac's architecture on the simulator
      if let identifier = ProcessInfo.processInfo.environment["SIMULATOR_MODEL_IDENTIFIER"] {
        return identifier
      }
    #endif
    var systemInfo = utsname()
    guard uname(&systemInfo) == 0 else {
      return nil
    }
    let machine = withUnsafeBytes(of: systemInfo.machine) { bytes in
      String(decoding: bytes.prefix(while: { $0 != 0 }), as: UTF8.self)
    }
    return machine.isEmpty ? nil : machine
  }

  private func isSimulator() -> Bool {
    #if targetEnvironment(simulator)
      return true
    #else
      return false
    #endif
  }

  /// Same value as `tauri::webview_version()`.
  private func webkitVersion() -> String? {
    return Bundle(identifier: "com.apple.WebKit")?.infoDictionary?["CFBundleVersion"] as? String
  }
}

@_cdecl("init_plugin_os")
public func initPlugin() -> Plugin {
  return OsPlugin()
}
