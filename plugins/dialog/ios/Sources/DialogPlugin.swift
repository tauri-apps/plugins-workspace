import UIKit
import WebKit
import Tauri
import SwiftRs

class DialogPlugin: Plugin {
		@objc public func showMessageDialog(_ invoke: Invoke) {
			let manager = self.manager
			let title = invoke.getString("title")
			guard let message = invoke.getString("message") else {
				invoke.reject("The `message` argument is required")
				return
			}
			let okButtonLabel = invoke.getString("okButtonLabel") ?? "OK"
			let cancelButtonLabel = invoke.getString("cancelButtonLabel") ?? "Cancel"
			
			DispatchQueue.main.async { [weak self] in
				let alert = UIAlertController(title: title, message: message, preferredStyle: UIAlertController.Style.alert)
				alert.addAction(UIAlertAction(title: cancelButtonLabel, style: UIAlertAction.Style.default, handler: { (_) -> Void in
					invoke.resolve([
						"value": false
					])
				}))
				alert.addAction(UIAlertAction(title: okButtonLabel, style: UIAlertAction.Style.default, handler: { (_) -> Void in
					invoke.resolve([
						"value": true
					])
				}))

				manager.viewController?.present(alert, animated: true, completion: nil)
			}
	}
}

@_cdecl("init_plugin_dialog")
func initPlugin(name: SRString, webview: WKWebView?) {
  Tauri.registerPlugin(webview: webview, name: name.toString(), plugin: DialogPlugin())
}