---
"barcode-scanner": patch
"barcode-scanner-js": patch
"biometric": patch
"biometric-js": patch
"clipboard-manager": patch
"clipboard-manager-js": patch
"deep-link": patch
"deep-link-js": patch
"dialog": patch
"dialog-js": patch
"geolocation": patch
"geolocation-js": patch
"haptics": patch
"haptics-js": patch
"nfc": patch
"nfc-js": patch
"notification": patch
"notification-js": patch
---

Fix the docs.rs build for Android: the build script now detects docs.rs through the `DOCS_RS` environment variable, since `cfg(docsrs)` is never set for build scripts.
