---
"opener": major
"opener-js": major
---

**Breaking:** Renamed the `reveal_item_in_dir` command to `reveal_items_in_dir`, so the `allow-reveal-item-in-dir` and `deny-reveal-item-in-dir` permissions are now `allow-reveal-items-in-dir` and `deny-reveal-items-in-dir`. The `revealItemInDir` JavaScript function and the `Opener::reveal_item_in_dir`/`Opener::reveal_items_in_dir` Rust APIs are unchanged.
