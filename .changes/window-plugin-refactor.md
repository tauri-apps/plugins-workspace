---
"window": "patch"
"window-js": "patch"
---

The window plugin is recieving a few changes to improve consistency and add new features:

- Removed `appWindow` variable from JS module, use `getCurrent` or `Window.getCurrent`.
- Removed `WindowManager`, `WebviewWindow` and `WebviewHandle` types and merged them into one `Window` type that matches the name of the rust window type.
- Added `Window.getCurrent` and `Window.getAll` which is a convenient method for `getCurrent` and `getAll` functions.
