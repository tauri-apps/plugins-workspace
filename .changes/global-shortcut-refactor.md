---
"global-shortcut": "patch"
---

**Breaking change** Refactored the plugin Rust APIs for better DX and flexibility:

- Changed `Builder::with_handler` to be a method instead of a static method, it will also be triggered for any and all shortcuts even if the shortcut is registered through JS.
- Added `Builder::with_shortcut` and `Builder::with_shortcuts` to register shortcuts on the plugin builder.
- Added `on_shortcut` and `on_all_shortcuts` to register shortcuts with a handler.
