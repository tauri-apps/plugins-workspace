---
fs: minor
fs-js: minor
---

Added the `scopeDroppedPaths` plugin config option (`plugins > fs > scopeDroppedPaths` in `tauri.conf.json`). Set it to `false` to stop adding the paths dropped onto any window to the fs scope, which otherwise gives every webview with an fs permission access to them (directories recursively). Defaults to `true`, the current behaviour.
