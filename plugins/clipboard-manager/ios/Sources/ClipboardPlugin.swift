// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import SwiftRs
import Tauri
import UIKit
import WebKit

enum WriteOptions: Codable {
  case plainText(text: String)
}

enum ReadClipData: Codable {
  case plainText(text: String)
}

class ClipboardPlugin: Plugin {
  @objc public func writeText(_ invoke: Invoke) throws {
    let options = try invoke.parseArgs(WriteOptions.self)
    let clipboard = UIPasteboard.general
    switch options {
    case .plainText(let text):
      clipboard.string = text
    default:
      invoke.unimplemented()
      return
    }
    invoke.resolve()

  }

  @objc public func readText(_ invoke: Invoke) throws {
    let clipboard = UIPasteboard.general
    if let text = clipboard.string {
      invoke.resolve(ReadClipData.plainText(text: text))
    } else {
      invoke.reject("Clipboard is empty")
    }
  }
}

@_cdecl("init_plugin_clipboard")
func initPlugin() -> Plugin {
  return ClipboardPlugin()
}
