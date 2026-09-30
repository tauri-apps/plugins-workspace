---
"deep-link": patch
---

Fixed an iOS regression where deep link URL types were generated from app links instead of custom URL schemes, which could prevent custom schemes from being registered correctly.

Also updated the generated iOS `CFBundleURLName` to use `$(PRODUCT_BUNDLE_IDENTIFIER).{scheme}`.

And added handling for `webcredentials:` in iOS entitlements

