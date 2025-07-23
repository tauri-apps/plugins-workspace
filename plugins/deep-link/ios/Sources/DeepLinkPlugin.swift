import SwiftRs
import Tauri
import UIKit
import WebKit

struct SetEventHandlerArgs: Decodable {
    let handler: Channel
}

@objc class DeepLinkPlugin: Plugin {
    private var webView: WKWebView?
    private var currentUrl: String?
    private var channel: Channel?

    // Called when the plugin is loaded
    @objc public override func load(webview: WKWebView) {
        super.load(webview: webview)
        self.webView = webview
        DeepLinkPlugin.instance = self
        
        // Check if app was launched via URL
        if let url = DeepLinkPlugin.launchedUrl {
            // If so, send to JS
            var event = JSObject()
            event["url"] = url.absoluteString
            self.channel?.send(event)
        }
    }

    // JS command: return current URL
    @objc public func getCurrent(_ invoke: Invoke) throws {
        var ret = JSObject()
        ret["url"] = self.currentUrl
        Logger.info("getCurrent: \(String(describing: ret))")
        invoke.resolve(ret)
    }

    // This command should not be added to the `build.rs` and exposed as it is only
    // used internally from the rust backend.
    @objc public func setEventHandler(_ invoke: Invoke) throws {
        let args = try invoke.parseArgs(SetEventHandlerArgs.self)
        self.channel = args.handler
        invoke.resolve()
    }

    // Static helpers to store/dispatch URLs
    static var instance: DeepLinkPlugin?
    static var launchedUrl: URL?
    static func handleOpenUrl(_ url: URL) {
        // Called from AppDelegate/SceneDelegate
        if let plugin = DeepLinkPlugin.instance {
            plugin.currentUrl = url.absoluteString
            if let ch = plugin.channel {
                var event = JSObject()
                event["url"] = url.absoluteString
                ch.send(event)
            }
        } else {
            // App not yet initialized; save for load()
            DeepLinkPlugin.launchedUrl = url
        }
    }
}

@UIApplicationMain
class AppDelegate: UIResponder, UIApplicationDelegate {
  func application(_ app: UIApplication, open url: URL, options: [UIApplication.OpenURLOptionsKey : Any] = [:]) -> Bool {
    DeepLinkPlugin.handleOpenUrl(url)
      Logger.info("AppDelegate: Opened URL: \(url)")
    return true
  }
}



// class SceneDelegate: UIResponder, UIWindowSceneDelegate {

//     var window: UIWindow?

//     // Called when a scene is being created and connected
//     func scene(_ scene: UIScene,
//                willConnectTo session: UISceneSession,
//                options connectionOptions: UIScene.ConnectionOptions) {

//         // Handle initial URL if app was launched via a deep link
//         if let urlContext = connectionOptions.urlContexts.first {
//             DeepLinkPlugin.handleOpenUrl(urlContext.url)
//         }
//     }

//     // Called when the app receives a deep link while already running
//     func scene(_ scene: UIScene, openURLContexts URLContexts: Set<UIOpenURLContext>) {
//         guard let url = URLContexts.first?.url else { return }
//         DeepLinkPlugin.handleOpenUrl(url)
//     }
// }

@_cdecl("init_plugin_deep_link")
func initPlugin() -> Plugin {
  return DeepLinkPlugin()
}

// import Tauri
// import UIKit
// import WebKit

// class PingArgs: Decodable {
//   let value: String?
// }

// class ExamplePlugin: Plugin {
//   @objc public func ping(_ invoke: Invoke) throws {
//     let args = try invoke.parseArgs(PingArgs.self)
//     invoke.resolve(["value": args.value ?? ""])
//   }
// }
