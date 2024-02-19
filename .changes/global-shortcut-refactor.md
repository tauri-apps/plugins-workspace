---
"global-shortcut": "patch"
---

**Breaking change** Refactored the plugin Rust APIs for better DX and flexibility:

- `GlobalShortcutExt::register` and `GlobalShortcutExt::register_all` now takes a second argument, which is a handler to be called when the shortcut is triggered.
- Add `GlobalShortcutExt::register_without_handler` and `GlobalShortcutExt::register_all_without_handler` that work the same as old APIs (i.e. register without a handler).
- Changed `Builder::with_handler` to be a method instead of a static method.
  ```rs
  Builder::new().with_handler().build()
  ```
  instead of
  ```rs
  Builder::with_handler().build()
  ```
- if using `Builder::with_handler`, it will be triggered for all shortcuts, even if the shortcut is registered through JS the new `GlobalShortcutExt::register`.
