---
updater: patch
updater-js: patch
---

On Windows, report why launching the installer failed from the `ShellExecuteW` return value (for example "Access is denied" when the user declines the UAC prompt) instead of an unrelated `GetLastError` message.
