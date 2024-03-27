---
"global-shortcut": "patch"
"global-shortcut-js": "patch"
---

Refactored the JS APIs:

- Enhanced `register` and `unregister` to take either a single shortcut or an array.
- Removed `registerAll` and `unregisterAll`, instead use `register` and `unregister` with an array.
