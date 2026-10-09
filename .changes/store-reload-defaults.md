---
"store": major
"store-js": major
---

**Breaking:** `Store::reload` (and `reload()` in JavaScript) now resets the store to its defaults before merging the on-disk state into it, so in-memory keys that are neither in the defaults nor on disk are dropped. Previously the on-disk state was merged into the current in-memory store. Use `reload_ignore_defaults` / `reload({ ignoreDefaults: true })` to fully match the on-disk state.
