---
"opener": "patch"
"opener-js": "patch"
---

Fixed `revealItemInDir` and the `reveal_item_in_dir` / `reveal_items_in_dir` Rust APIs panicking on Linux and the BSDs with `Cannot start a runtime from within a runtime` when zbus is built with its `tokio` feature and the call happens inside an async runtime. The blocking D-Bus calls now run on a dedicated thread.
