---
"deep-link": patch
"deep-link-js": patch
---

Fixed the iOS `CFBundleURLTypes` generation: every configured custom URL scheme is now registered, including schemes listed next to `https`/`http` in the same `mobile` entry, and web link entries no longer produce an empty URL type.

The generated iOS `CFBundleURLName` is now `$(PRODUCT_BUNDLE_IDENTIFIER).{scheme}`.
