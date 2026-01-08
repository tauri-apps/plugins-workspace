---
"fs": patch
"fs-js": patch
---

Fix off by one error in the implementation of readTextFileLines causing all lines to end with an (additional) null byte.
Issue: [#3154](https://github.com/tauri-apps/plugins-workspace/issues/3154)
PR: [#3155](https://github.com/tauri-apps/plugins-workspace/pull/3155)
