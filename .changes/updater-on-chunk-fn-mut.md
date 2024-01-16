---
"updater": "patch"
---

**Breaking change**: Changed `Update::download` and `Update::download_and_install` first argument to take `FnMut` instead of just `Fn`.
