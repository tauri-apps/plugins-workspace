---
updater: patch
updater-js: patch
---

Calling `Update.download()` again now releases the bytes of the previous download instead of keeping them in memory until the webview is destroyed.
