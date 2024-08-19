---
barcode-scanner: patch
clipboard-manager: patch
deep-link: patch
global-shortcut: patch
window-state: patch
---

Fixed an issue that caused multi-word IIFE names to not be formatted correctly. For example the `barcode-scanner` was defined as `window.__TAURI_PLUGIN_CLIPBOARDMANAGER__` instead of `window.__TAURI_PLUGIN_CLIPBOARD_MANAGER__`.
