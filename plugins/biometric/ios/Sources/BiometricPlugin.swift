// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import SwiftRs
import Tauri
import UIKit
import WebKit

class BiometricPlugin: Plugin {
  @objc public func ping(_ invoke: Invoke) throws {
    let value = invoke.getString("value")
    invoke.resolve(["value": value as Any])
  }
}

@_cdecl("init_plugin_biometric")
func initPlugin() -> Plugin {
  return BiometricPlugin()
}
