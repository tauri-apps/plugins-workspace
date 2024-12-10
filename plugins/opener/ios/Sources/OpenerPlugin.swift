// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation
import SwiftRs
import Tauri
import UIKit
import WebKit

class OpenerPlugin: Plugin {
  @objc public func open(_ invoke: Invoke) throws {
    do {
      let urlString = try invoke.parseArgs(String.self)
      if let url = URL(string: urlString) {
        if #available(iOS 10, *) {
          UIApplication.shared.open(url, options: [:])
        } else {
          UIApplication.shared.openURL(url)
        }
      }
      invoke.resolve()
    } catch {
      invoke.reject(error.localizedDescription)
    }
  }
}

@_cdecl("init_plugin_opener")
func initPlugin() -> Plugin {
  return OpenerPlugin()
}
