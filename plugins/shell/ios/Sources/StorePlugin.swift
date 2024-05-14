// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import Foundation

import SwiftRs
import Tauri
import UIKit
import WebKit


struct SaveStore: Codable {
    let store: String
    let cache: [String: JSON]
}

class StorePlugin: Plugin {

    @objc public func load(_ invoke: Invoke) throws {
        do {
            let urlString = try invoke.parseArgs(String.self)
            let url = URL(string: urlString)
            UIApplication.shared.openURL(url)
            invoke.resolve()
        } catch {
            invoke.reject(error.localizedDescription)
        }
    }
}


@_cdecl("init_plugin_shell")
func initPlugin() -> Plugin {
    return ShellPlugin()
}
