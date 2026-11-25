// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import AVFoundation
import SwiftRs
import Tauri
import UIKit
import WebKit

private let errorNotAvailable = "The torch is not available on this device"

class TorchPlugin: Plugin {
  // The back camera, whose flash unit is the torch. Locking it for configuration also works while a
  // capture session (e.g. the barcode scanner) is running on it.
  private func torchDevice() -> AVCaptureDevice? {
    let device =
      AVCaptureDevice.default(.builtInWideAngleCamera, for: .video, position: .back)
      ?? AVCaptureDevice.default(for: .video)
    guard let device = device, device.hasTorch else { return nil }
    return device
  }

  private func setTorch(_ invoke: Invoke, enabled: Bool) -> Bool {
    let action = enabled ? "enable" : "disable"
    guard let device = torchDevice() else {
      invoke.reject(errorNotAvailable)
      return false
    }
    // false while the device is overheating, among others
    guard device.isTorchAvailable else {
      invoke.reject("Failed to \(action) the torch: the torch is temporarily unavailable")
      return false
    }
    do {
      try device.lockForConfiguration()
      defer { device.unlockForConfiguration() }
      if enabled {
        try device.setTorchModeOn(level: AVCaptureDevice.maxAvailableTorchLevel)
      } else {
        device.torchMode = .off
      }
      return true
    } catch {
      invoke.reject("Failed to \(action) the torch: \(error.localizedDescription)")
      return false
    }
  }

  @objc public func enable(_ invoke: Invoke) {
    if setTorch(invoke, enabled: true) {
      invoke.resolve()
    }
  }

  @objc public func disable(_ invoke: Invoke) {
    if setTorch(invoke, enabled: false) {
      invoke.resolve()
    }
  }

  @objc public func toggle(_ invoke: Invoke) {
    let enabled = !(torchDevice()?.isTorchActive ?? false)
    if setTorch(invoke, enabled: enabled) {
      invoke.resolve(["enabled": enabled])
    }
  }

  @objc public func isAvailable(_ invoke: Invoke) {
    invoke.resolve(["available": torchDevice() != nil])
  }

  @objc public func isEnabled(_ invoke: Invoke) {
    invoke.resolve(["enabled": torchDevice()?.isTorchActive ?? false])
  }
}

@_cdecl("init_plugin_torch")
public func initPlugin() -> Plugin {
  return TorchPlugin()
}
