---
updater: patch
updater-js: patch
---

On Linux, reject an AppImage update that is neither an AppImage nor a compressed AppImage (or a compressed AppImage when the `zip` feature is disabled) before touching the installed app. Such payloads were written over the AppImage as-is, corrupting the app.
