---
"shell": patch
"shell-js": patch
---

Removed the webview script that sent clicks on `<a target="_blank">` links to the removed `open` command. Since that command was removed, the script made those links do nothing, even when `tauri-plugin-opener` was registered. Opener now handles them again.
