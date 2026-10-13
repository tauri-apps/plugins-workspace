---
updater: patch
updater-js: patch
---

On Linux, try the next temporary directory location (the user cache directory, then the directory holding the AppImage) when a temporary directory cannot be created in the previous one, instead of failing the AppImage update right away. Also avoids a panic when the app path has no parent directory.
