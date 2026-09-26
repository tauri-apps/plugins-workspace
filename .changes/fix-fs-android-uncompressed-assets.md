---
fs: patch
fs-js: patch
---

Fixed reading uncompressed Android assets (`$RESOURCE` files such as images or audio that are stored uncompressed in the APK) returning the bytes of the APK instead of the asset. Assets are now always read from a copy in the app cache, like compressed ones.
