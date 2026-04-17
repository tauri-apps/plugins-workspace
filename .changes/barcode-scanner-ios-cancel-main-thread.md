---
"barcode-scanner": patch
"barcode-scanner-js": patch
---

Fix a crash on iOS when `cancel()` is invoked. The pending camera
teardown (`removeFromSuperview`, webView background/opacity updates)
was running on the background dispatch queue that Tauri uses to
deliver plugin commands, which caused UIKit to abort the app with an
`NSAssertionHandler` failure inside `_didMoveFromWindow:toWindow:`.
The cancel handler now dispatches the UIKit work to the main thread,
matching the existing pattern used by `scan()` and `openAppSettings()`.
