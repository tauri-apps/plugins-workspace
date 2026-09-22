---
"shell": major
"shell-js": major
---

**Breaking:** Removed the `open` API, which had been deprecated since v2.1.0 in favor of `tauri-plugin-opener`. This removes the `open` command and its `allow-open`/`deny-open` permissions, the `plugins > shell > open` configuration (`tauri_plugin_shell::init()` now returns `TauriPlugin<R>`), the `Shell::open` method and the `tauri_plugin_shell::open` module, the `Error::UnknownProgramName` variant, and the `open` JavaScript function. The `shell:default` permission set now grants nothing. Since `open` was the only mobile functionality, the Android and iOS plugins were removed as well.
