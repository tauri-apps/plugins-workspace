---
"store": patch
---

Add a setting `auto_save` to enable a store to debounce save on modification (on calls like set, clear, delete, reset)

**Breaking change**: `with_store` now takes one more parameter `auto_save: Option<Duration>`
