---
"barcode-scanner": patch
"barcode-scanner-js": patch
---

Fix `cancel()` on Android leaving the pending `scan()` promise unsettled: `destroy()` cleared the saved invoke before it was rejected, so the `"cancelled"` rejection never reached the caller. The pending scan is now rejected before the camera is torn down, matching iOS.
