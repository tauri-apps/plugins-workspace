---
"global-shortcut": "patch"
---

Refactored the Rust APIs:

- Renamed `GlobalShortcut::on_all_shortcuts` to `GlobalShortcut::on_shortcuts`
- Renamed `GlobalShortcut::register_all` to `GlobalShortcut::register_multiple`
- Changed `GlobalShortcut::unregister_all` behavior to remove all registerd shortcuts.
- Added `GlobalShortcut::unregister_multiple` to register a list of shortcuts (old behavior of `unregister_all`).
