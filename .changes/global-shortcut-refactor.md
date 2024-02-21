---
"global-shortcut": "patch"
---

**Breaking change** Refactored the plugin Rust APIs for better DX and flexibility:

- `GlobalShortcutExt::register` and `GlobalShortcutExt::register_all` now takes a second argument, which is a optional handler to be called when the shortcut is triggered.
- Changed `Builder::with_handler` to be a method instead of a static method, it will also be triggered for any and all shortcuts.
  ```rs
  Builder::new().with_handler().build()
  ```
  instead of
  ```rs
  Builder::with_handler().build()
  ```
- if using `Builder::with_handler`, it will be triggered for all shortcuts, even if the shortcut is registered through JS or through `GlobalShortcutExt::register`.
