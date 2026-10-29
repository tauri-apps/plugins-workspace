---
"store": patch
"store-js": patch
---

Fixed a deadlock between loading a store and closing a store at the same time, which could freeze the main thread for good. `StoreExt::get_store` and the save on exit took the same two locks in the wrong order too. Store files are also no longer read while the plugin's stores lock is held.
