import UIKit
import WebKit
import Tauri
import SwiftRs

class NotificationPlugin: Plugin {
	@objc public func requestPermission(_ invoke: Invoke) throws {
		invoke.resolve(["permissionState": "granted"])
	}

	@objc public func permissionState(_ invoke: Invoke) throws {
		invoke.resolve(["permissionState": "granted"])
	}

	@objc public func notify(_ invoke: Invoke) throws {
		// TODO
		invoke.resolve()
	}
}

@_cdecl("init_plugin_notification")
func initPlugin(name: SRString, webview: WKWebView?) {
	Tauri.registerPlugin(webview: webview, name: name.toString(), plugin: NotificationPlugin())
}
