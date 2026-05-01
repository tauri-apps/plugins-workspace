---
store: patch
store-js: patch
---

Fix a deadlock when calling `Store::set` while exiting (on `RunEvent::Exit`)
