---
upload: patch
upload-js: patch
---

Fix `download` and `upload` locks main thread on Android.
Use Tokio to spawn task when invoking commands.