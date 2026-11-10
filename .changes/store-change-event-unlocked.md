---
store: patch
store-js: patch
---

Send `store://change` events after the store is unlocked. A Rust listener that read the store, directly or through a logger reading its level from it, hung the `set`, `delete`, `clear` or `reset` call that triggered it. `reset` also sent its events before applying the defaults.
