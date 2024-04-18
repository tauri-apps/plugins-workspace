---
"clipboard-manager": "patch"
---

Refactored the clipboard Rust APIs for more clarity and consistency:

- Changed `Clipboard::write_text` to take a string type instead of an enum.
- Changed `Clipboard::read_text` to return a string type instead of an enum.
- Changed `Clipboard::write_html` to take 2 string arguments instead of an enum.
- Changed `Clipboard::write_image` to take a reference to a `tauri::Image` instead of an enum.
- Removed `ClipKind` and `ClipboardContents` enums.
