---
"updater": patch
"updater-js": patch
---

On macOS, a failed install no longer deletes the installed app. The bundles are swapped atomically where the file system supports it; otherwise the previous app is restored, or kept as `<name> (previous version).app` and reported through the new `Error::PreviousAppNotRestored`. Also fixes installing apps on a volume other than the temp directory's, and the bundle root's permissions (`0755`).
