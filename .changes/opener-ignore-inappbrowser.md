---
opener: patch
opener-js: patch
---

Properly ignore `with: inAppBrowser` on desktop. This prevents an issue were `open_url` seamingly did nothing on desktop.
