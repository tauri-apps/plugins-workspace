---
"global-shortcut": "patch"
"global-shortcut-js": "patch"
---

Refactored APIs to introduce new pressed and released events:

- Added `ShortcutEvent` and `ShortcutState` types in Rust.
- Changed the handler function passed to `GlobalShortcut::on_shortcut`, `GlobalShortcut::on_all_shortcuts` and `Builder::with_handler` to take a 3rd argument of type `ShortcutEvent`.
- Added `ShortcutEvent` interface in JS.
- Changed `ShortcutHandler` type alias (which affects the JS `register` and `registerAll` APIs) to take `ShortcutEvent` instead of a string.
