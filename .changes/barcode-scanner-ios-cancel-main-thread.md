---
"barcode-scanner": patch
"barcode-scanner-js": patch
---

Fixed a crash on iOS when `cancel()` is invoked by running the cancel handler on the main thread.
