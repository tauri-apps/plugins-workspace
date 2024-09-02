// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import LocalAuthentication
import SwiftRs
import Tauri
import UIKit
import WebKit


struct SharesheetOptions: Decodable {
  let text: String
  let title: String?
}

class SharesheetPlugin: Plugin {
  public override func load(webview: WKWebView) {
  }

  @objc func shareText(_ invoke: Invoke) throws {
    let args = try invoke.parseArgs(SharesheetOptions.self)

    let content;
    if args.text.hasPrefix("http://") || args.text.hasPrefix("https://") {
      content = Url(args.text)
    } else {
      content = Text(args.text)
    }

    let title;
    if args.title != nil {
      title = Text(args.title)
    }

    ShareLink(item: content, message: title)
  }
}

@_cdecl("init_plugin_sharesheet")
func initPlugin() -> Plugin {
  return SharesheetPlugin()
}
