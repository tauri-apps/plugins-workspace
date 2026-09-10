---
"updater": patch
"updater-js": patch
---

On macOS, a failed install no longer deletes the installed app. The previous bundle is moved back into place when the new one cannot be moved in, the privileged fallback moves the previous bundle aside instead of deleting it first, the update is refused up front when the temp directory is on another volume, and the installed bundle root is `0755` rather than the temp directory's `0700`.
