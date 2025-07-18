---
upload: minor
upload-js: minor
---

Fix `download` and `upload` locks main thread on Android.
Use Tokio to spawn task when invoking commands.